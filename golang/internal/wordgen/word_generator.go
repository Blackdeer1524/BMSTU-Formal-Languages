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
	rg         []RegexesWithWords
}

type RegexesWithWords struct {
	RegexBefore string
	RegexAfter  string
	Words       []string
}

func GenerateWordsForRegexes(
	regexes []string,
	countWords, maxDumpSize int,
) ([]RegexesWithWords, error) {
	rwws := make([]RegexesWithWords, len(regexes))

	for i := 0; i < len(rwws); i++ {
		rww, err := GenerateWordsForRegex(regexes[i], countWords, maxDumpSize)
		if err != nil {
			return nil, err
		}
		rwws[i] = *rww
	}

	return rwws, nil
}

func GenerateWordsForBenchmarkRegexes(
	rwws []RegexesWithWords,
) ([]RegexesWithWords, error) {

	addZtoAndWords(&rwws)

	return rwws, nil
}

func addZtoAndWords(rwws *[]RegexesWithWords) {
	for i := 0; i < len(*rwws); i++ {
		for j := 0; j < len((*rwws)[i].Words); j++ {
			(*rwws)[i].Words[j] += "Z"
		}
	}
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

func GenerateWordsForRegex(regex string, countWords, maxDumpSize int) (*RegexesWithWords, error) {
	tree, pErr := parser.ParseRegex(regex)
	if pErr != nil {
		return nil, fmt.Errorf("can't parse regex: %w", pErr)
	}

	automaton := gluskov.BuildMachine(tree, regex)

	// Визуализация автомата
	//err := automaton.GetDotMachine()
	//if err != nil {
	//	return nil, err
	//}

	// Визуализация дерева разбора
	//err = parser.ParseRegexInDot(regex)
	//if err != nil {
	//	return nil, err
	//}

	loops := loop.FindCycles(automaton)
	letterLoop := loop.TranslateLoops(loops)

	org := New(regex, countWords, maxDumpSize, automaton, letterLoop)

	for i := 0; i < countWords; i++ {
		org.Words[i] = org.DfsBuildWord(automaton, letterLoop)
	}

	return &RegexesWithWords{
		Words:       org.Words,
		RegexBefore: regex,
	}, nil
}

func (org *OneRegexpGenerator) DfsBuildWord(
	machine *gluskov.Machine,
	loops loop.StateLoopToString,
) string {
	visited := make(map[gluskov.State]bool)
	word := strings.Builder{}

	dfs(*machine, gluskov.State(0), loops, &visited, &word, org.MaxDumpSize)

	return word.String()
}

func dfs(
	m gluskov.Machine,
	currentState gluskov.State,
	loops loop.StateLoopToString,
	visited *map[gluskov.State]bool,
	word *strings.Builder,
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

	if len(m.Transitions[currentState]) > 0 {
		randomLetter, randomStates := randomStatesTransition(m.Transitions[currentState])
		(*word).WriteRune(randomLetter)
		for _, nextState := range randomStates {
			if !(*visited)[nextState] {
				dfs(m, nextState, loops, visited, word, maxDumpSize)
			}
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

func dumpWord(word *strings.Builder, pumping string, maxPumpingCount int) int {
	pumpingCount := rand.Intn(maxPumpingCount)
	(*word).WriteString(strings.Repeat(pumping, pumpingCount))
	return pumpingCount
}
