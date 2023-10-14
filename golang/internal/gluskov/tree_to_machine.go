package gluskov

import (
	"fmt"
	"regexp/syntax"
)

type State int
type StateTransitions map[rune][]State

type Machine struct {
	StartState   int
	FinalStates  []State
	Transitions  map[State]StateTransitions
	StateCounter int
}

func BuildMachine(st *syntax.Regexp) *Machine {
	machine := &Machine{
		StartState:   0,
		FinalStates:  make([]State, 0),
		Transitions:  make(map[State]StateTransitions),
		StateCounter: 1,
	}

	machine.handleRegex(st, State(machine.StartState), true)

	return machine
}

func (m *Machine) handleRegex(node *syntax.Regexp, currentState State, isFinal bool) []State {
	switch node.Op {
	case syntax.OpLiteral:
		return m.handleLiteral(currentState, node, isFinal)
	case syntax.OpConcat:
		return m.handleConcat(currentState, node, isFinal)
	case syntax.OpAlternate:
		return m.handleAlternate(currentState, node, isFinal)
	case syntax.OpStar:
		return m.handleStar(currentState, node, isFinal)
	case syntax.OpCapture:
		return m.handleCapture(currentState, node, isFinal)
	case syntax.OpCharClass:
		return m.handleCharClass(currentState, node, isFinal)
	}
	fmt.Println("вот кто вышел за case:", node.Op)
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
	m.FinalStates = append(m.FinalStates, s)
}

func (m *Machine) GetRuneBetweenStates(left, right State) (rune, State) {
	if right == 0 {
		return 'я', 0
	}
	for r, states := range m.Transitions[left] {
		for _, s := range states {
			if s == right {
				return r, s
			}
		}
	}
	return m.GetRuneBetweenStates(left, right-1)
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

func (m *Machine) handleConcat(currentState State, node *syntax.Regexp, isFinal bool) []State {

	leftState := m.handleRegex(node.Sub[0], currentState, false)
	rightState := m.handleRegex(node.Sub[1], leftState[0], isFinal)

	transition, newRight := m.GetRuneBetweenStates(leftState[0], rightState[0])

	for i := 1; i < len(leftState); i++ {
		m.addTransition(leftState[i], newRight, transition)
	}

	return rightState
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

func (m *Machine) handleStar(currentState State, node *syntax.Regexp, isFinal bool) []State {
	endStarState := m.handleRegex(node.Sub[0], currentState, isFinal)

	translations, _ := m.GetRuneBetweenStates(currentState, currentState+1)

	m.addTransition(endStarState[0], currentState, translations)
	return endStarState
}

func (m *Machine) handleCapture(currentState State, node *syntax.Regexp, isFinal bool) []State {
	if len(node.Sub) != 1 {
		panic("Длина node.Sub в захвате не равна 1 -> такой случай я не рассматривал")
	}
	return m.handleRegex(node.Sub[0], currentState, isFinal)
}

func (m *Machine) handleCharClass(currentState State, node *syntax.Regexp, isFinal bool) []State {
	leftState := m.addState()
	m.addTransition(currentState, leftState, node.Rune[0])

	rightState := m.addState()
	m.addTransition(currentState, rightState, node.Rune[1])

	if isFinal {
		m.addFinal(leftState)
		m.addFinal(rightState)
	}
	return []State{leftState, rightState}
}
