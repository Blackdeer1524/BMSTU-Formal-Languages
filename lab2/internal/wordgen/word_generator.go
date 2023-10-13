package wordgen

import (
	"fmt"
	"math/rand"
	"strings"

	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/gluskov"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/loop"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/parser"
)

type ManyRegexpWordGenerator struct {
	countRegex int
	rg         []OneRegexpGenerator
}

type OneRegexpGenerator struct {
	Regex       string
	CountWord   int
	MaxDumpSize int
	Words       []string
	machine     *gluskov.Machine
	letterLoop  loop.StateLoopToString
}

func New(
	regex string, countWord, maxDumpSize int,
	machine *gluskov.Machine,
	letterLoop loop.StateLoopToString,
) *OneRegexpGenerator {
	return &OneRegexpGenerator{
		Words:       make([]string, countWord),
		CountWord:   countWord,
		MaxDumpSize: maxDumpSize,
		Regex:       regex,
		machine:     machine,
		letterLoop:  letterLoop,
	}
}

func GenerateWordsForRegex(regex string, countWords, maxDumpSize int) ([]string, error) {
	tree, pErr := parser.ParseRegex(regex)
	if pErr != nil {
		return nil, fmt.Errorf("can't parse regex %w", pErr)
	}

	automaton := gluskov.BuildMachine(tree)

	err := automaton.GetDotMachine()
	if err != nil {
		return nil, err
	}

	err = parser.ParseRegexInDot(regex)
	if err != nil {
		return nil, err
	}

	loops := loop.FindCycles(automaton)
	letterLoop := loop.TranslateLoops(loops, automaton)

	org := New(regex, countWords, maxDumpSize, automaton, letterLoop)

	for i := 0; i < countWords; i++ {
		org.Words[i] = org.DfsBuildWord(automaton, letterLoop)
	}

	return org.Words, nil
}

func (org *OneRegexpGenerator) DfsBuildWord(
	machine *gluskov.Machine,
	loops loop.StateLoopToString,
) string {
	visited := make(map[gluskov.State]bool)
	word := ""

	dfs(*machine, gluskov.State(0), loops, &visited, &word, org.MaxDumpSize)

	return word
}

func dfs(
	m gluskov.Machine,
	currentState gluskov.State,
	loops loop.StateLoopToString,
	visited *map[gluskov.State]bool,
	word *string,
	maxDumpSize int,
) {
	if l, ok := loops[currentState]; ok {
		randomLoop := l[rand.Intn(len(l))]
		pumpCount := dumpWord(word, randomLoop.AlpLoop, maxDumpSize)
		if pumpCount > 0 {
			currentState = randomLoop.FinalState
		}
	}

	(*visited)[currentState] = true

	randomLetter, randomStates := randomStatesTransition(m.Transitions[currentState])
	*word += string(randomLetter)
	for _, nextState := range randomStates {
		if !(*visited)[nextState] {
			dfs(m, nextState, loops, visited, word, maxDumpSize)
		}
	}
}

func randomStatesTransition(st gluskov.StateTransitions) (rune, []gluskov.State) {
	randomLetterCount := rand.Intn(len(st))
	counter := 0
	for f, nextStates := range st {
		if randomLetterCount == counter {
			return f, nextStates
		}
		counter++
	}
	return 0, nil
	// break
	// Небольшое README, почему здесь break
	// for range по map`е в условиях go runtime будет рандомно выбирать ключ из мапы.
	// То есть это законный способ получать случайные пути в автомате.
	// То есть я беру первый попавшийся путь в автомате
	// Пока искал статью, которой можно это подтвердить, набрел на интересную дискуссию:
	// https://www.reddit.com/r/golang/comments/kiees6/how_to_pick_a_random_value_from_a_map/
	// То есть поведение не рандомное, а "implementation-specific", и полагаться на это не стоит
	// Тут есть способ интереснее и более рандомный, мне это нравится.
	// Оставлю это как историю
}

func dumpWord(word *string, pumping string, maxPumpingCount int) int {
	pumpingCount := rand.Intn(maxPumpingCount)
	*word += strings.Repeat(pumping, pumpingCount)
	return pumpingCount
}
