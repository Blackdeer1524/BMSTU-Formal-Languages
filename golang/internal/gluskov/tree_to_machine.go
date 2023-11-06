package gluskov

import (
	"fmt"
	"log"
	"regexp"
	"regexp/syntax"
	"strconv"
	"strings"
)

type State int
type StateTransitions map[rune][]State

type Machine struct {
	StartState   int
	FinalStates  map[State]struct{}
	Transitions  map[State]StateTransitions
	StateCounter int
}

func BuildMachine(st *syntax.Regexp, regex string) *Machine {
	machine := &Machine{
		StartState:   0,
		FinalStates:  make(map[State]struct{}),
		Transitions:  make(map[State]StateTransitions),
		StateCounter: 1,
	}

	machine.handleRegex(st, State(machine.StartState), true)

	machine.findAllFinalStates("^" + regex + "$")

	return machine
}

func (m *Machine) handleRegex(node *syntax.Regexp, currentState State, isFinal bool) []State {
	switch node.Op {
	case syntax.OpLiteral:
		return m.handleLiteral(currentState, node, isFinal)
	case syntax.OpConcat:
		return m.handleConcatMulti(currentState, node)
	case syntax.OpAlternate:
		return m.handleAlternate(currentState, node, isFinal)
	case syntax.OpStar:
		return m.handleStar(currentState, node, isFinal)
	case syntax.OpCapture:
		return m.handleCapture(currentState, node, isFinal)
	case syntax.OpCharClass:
		return m.handleCharClass(currentState, node, isFinal)
	}
	log.Println("ERROR: вот кто вышел за case:", node.Op)
	return []State{currentState}
}

func (m *Machine) addTransition(fromState, toState State, symbol rune) {
	if _, exists := m.Transitions[fromState]; !exists {
		m.Transitions[fromState] = make(StateTransitions)
	}
	m.Transitions[fromState][symbol] = append(m.Transitions[fromState][symbol], toState)
}

func (m *Machine) addState() State {
	newState := State(m.StateCounter)
	m.StateCounter++
	return newState
}

func (m *Machine) addFinal(s State) {
	m.FinalStates[s] = struct{}{}
}

func (m *Machine) handleLiteral(currentState State, node *syntax.Regexp, isFinal bool) []State {
	for _, symbol := range node.Rune {
		nextState := m.addState()
		m.addTransition(currentState, nextState, symbol)
		currentState = nextState
	}
	if isFinal {
		m.addFinal(currentState)
	}
	return []State{currentState}
}

func (m *Machine) handleAlternate(currentState State, node *syntax.Regexp, isFinal bool) []State {
	leftState := m.handleRegex(node.Sub[0], currentState, isFinal)
	rightState := m.handleRegex(node.Sub[1], currentState, isFinal)

	if isFinal {
		m.addFinal(leftState[0])
		m.addFinal(rightState[0])
	}
	return []State{leftState[0], rightState[0]}
}

func (m *Machine) handleConcatMulti(currentState State, node *syntax.Regexp) []State {

	beforeState := m.handleRegex(node.Sub[0], currentState, false)

	nextState := make([]State, 0)
	for i := 1; i < len(node.Sub); i++ {
		nextState = m.concatRange(beforeState, node.Sub[i], false)
		beforeState = nextState
	}

	return nextState
}

func (m *Machine) concatRange(leftState []State, node *syntax.Regexp, isFinal bool) []State {
	currentTransitionsBefore := getNewStateTransitionsObject(m.Transitions[leftState[0]])

	rightState := m.handleRegex(node, leftState[0], isFinal)
	currentTransitionsAfter := getNewStateTransitionsObject(m.Transitions[leftState[0]])

	newTransitions, err := getLetterTransitionAndNextState(currentTransitionsBefore, currentTransitionsAfter)
	if err != nil {
		log.Println("Ошибка в конкатенации", err)
	}

	for i := 1; i < len(leftState); i++ {
		for j := 0; j < len(newTransitions); j++ {
			m.addTransition(leftState[i], newTransitions[j].newState, newTransitions[j].letter)
		}
	}

	return rightState
}

type oneTransition struct {
	letter   rune
	newState State
}

func (m *Machine) handleStar(currentState State, node *syntax.Regexp, isFinal bool) []State {

	currentTransitionsBefore := getNewStateTransitionsObject(m.Transitions[currentState])

	endStarState := m.handleRegex(node.Sub[0], currentState, isFinal)

	currentTransitionsAfter := getNewStateTransitionsObject(m.Transitions[currentState])

	newTransitions, err := getLetterTransitionAndNextState(currentTransitionsBefore, currentTransitionsAfter)
	if err != nil {
		log.Println("Ошибка в звезде Клини", err)
	}

	for i := 0; i < len(newTransitions) && i < len(endStarState); i++ {
		m.addTransition(endStarState[i], newTransitions[i].newState, newTransitions[i].letter)
	}

	return endStarState
}

type finderStateToHash map[rune]string

func getNewStateTransitionsObject(st StateTransitions) finderStateToHash {
	fsh := make(finderStateToHash)

	for letter, states := range st {
		fsh[letter] = getHash(states)
	}
	return fsh
}

func getHash(states []State) string {
	sb := strings.Builder{}
	for _, s := range states {
		sb.WriteString(strconv.Itoa(int(s)))
		sb.WriteRune('_')
	}
	return sb.String()
}

// Как это работает:
// До handle у нас есть какое-то состояние автомата
// После handle в автомате появляется новая ветка состояний
// Соответственно в currentState появляется какой-то новый переход
// Его я и хочу получить.
// Если его нет - какая-то ошибка, он должен быть
// Должен возвращать не state, а []state, потому что из альтернативы под звездой может вернуться несколько состояний
func getLetterTransitionAndNextState(before, after finderStateToHash) ([]oneTransition, error) {
	// в состоянии before у нас на какое-то число состояний меньше
	ot := make([]oneTransition, 0)
	for letter, hashAfter := range after {
		// Нашелся новый литерал в переходе
		if hashBefore, ok := before[letter]; !ok {
			newTransitions, err := convertTransitions(letter, hashAfter)
			if err != nil {
				return nil, fmt.Errorf("can't parse newState in new letter %w", err)
			}
			ot = append(ot, newTransitions...)
		} else {
			// Не новый литерал в переходе, но поменялся массив состояний
			if hashBefore != hashAfter {
				newState := findDifferenceBetweenArray(letter, hashBefore, hashAfter)
				ot = append(ot, newState...)
			}
		}
	}

	return ot, nil
}

func convertTransitions(letter rune, hash string) ([]oneTransition, error) {

	ot := make([]oneTransition, 0)

	hashArr := strings.Split(hash[:len(hash)-1], "_")

	for _, ha := range hashArr {
		newState, err := strconv.Atoi(ha)
		if err != nil {
			log.Println("ошибка в convert transitions", err)
		}
		ot = append(ot, oneTransition{letter: letter, newState: State(newState)})
	}
	return ot, nil
}

func findDifferenceBetweenArray(letter rune, before, after string) []oneTransition {

	ot := make([]oneTransition, 0)

	beforeStr := strings.Split(before[:len(before)-1], "_")
	afterStr := strings.Split(after[:len(after)-1], "_")

	for i := len(beforeStr); i < len(afterStr); i++ {
		newState, err := strconv.Atoi(afterStr[i])
		if err != nil {
			log.Println("problem in star letter already exist")
		}
		ot = append(ot, oneTransition{letter: letter, newState: State(newState)})
	}

	return ot
}

func (m *Machine) handleCapture(currentState State, node *syntax.Regexp, isFinal bool) []State {
	if len(node.Sub) != 1 {
		panic("Длина node.Sub в захвате не равна 1 -> такой случай я не рассматривал")
	}
	return m.handleRegex(node.Sub[0], currentState, isFinal)
}

// Нашлось странное поведение:
// для regex := "(f|h)*" CharClass содержит [fh]
// Но node.Rune() содержит [102 102 104 104] -> откуда и почему - неизвестно
// Для f|h то же самое - хотя раньше было по-другому. Какой-то баг внутри regexp.....
func (m *Machine) handleCharClass(currentState State, node *syntax.Regexp, isFinal bool) []State {
	firstLetter, secondLetter := 0, 1
	if len(node.Rune) == 4 {
		firstLetter++
		secondLetter++
	}

	leftState := m.addState()
	m.addTransition(currentState, leftState, node.Rune[firstLetter])

	rightState := m.addState()
	m.addTransition(currentState, rightState, node.Rune[secondLetter])

	if isFinal {
		m.addFinal(leftState)
		m.addFinal(rightState)
	}
	return []State{leftState, rightState}
}

type finalStateWord struct {
	word  string
	state State
}

func (m *Machine) generateAllWords() []finalStateWord {
	words := make([]finalStateWord, 0)
	visited := make(map[State]bool)
	m.generateWordsFromState(State(m.StartState), "", visited, &words)
	return words
}

func (m *Machine) generateWordsFromState(
	currentState State,
	currentWord string,
	visited map[State]bool,
	words *[]finalStateWord,
) {
	visited[currentState] = true

	*words = append(*words, finalStateWord{currentWord, currentState})

	transitions := m.Transitions[currentState]

	for symbol, nextStates := range transitions {
		for _, nextState := range nextStates {
			nextWord := currentWord + string(symbol)

			if !visited[nextState] {
				m.generateWordsFromState(nextState, nextWord, visited, words)
			}
		}
	}

	visited[currentState] = false
}

func (m *Machine) findAllFinalStates(regex string) {
	wordsWithStates := m.generateAllWords()

	for _, ww := range wordsWithStates {
		m.finalStateCheck(regex, ww)
	}
}

func (m *Machine) finalStateCheck(regex string, fsw finalStateWord) {
	if checkWordInRegex(regex, fsw.word) {
		m.addFinal(fsw.state)
	}
}

func checkWordInRegex(regex string, word string) bool {
	ok, err := regexp.MatchString(regex, word)
	if err != nil {
		log.Println("error in match final state", err)
	}
	return ok
}
