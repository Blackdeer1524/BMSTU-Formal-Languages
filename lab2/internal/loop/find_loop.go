package loop

import (
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/gluskov"
)

type dfsParam struct {
	machine *gluskov.Machine
	cycles  [][]gluskov.State
	visited map[gluskov.State]bool
	path    []gluskov.State
}

func FindCycles(machine *gluskov.Machine) [][]gluskov.State {

	dfsp := newDfsParam(machine)

	// вообще у меня одна компонента связности и мне бы хватило одного вызова dfs от стартового состояния, но да ладно
	// на непредвиденный случай оставлю так
	for i := 0; i < machine.StateCounter; i++ {
		if !dfsp.visited[gluskov.State(i)] {
			dfsp.dfsFinderLoop(gluskov.State(i))
		}
	}

	return dfsp.cycles
}

func newDfsParam(machine *gluskov.Machine) *dfsParam {
	return &dfsParam{
		machine: machine,
		cycles:  make([][]gluskov.State, 0),
		visited: make(map[gluskov.State]bool),
		path:    make([]gluskov.State, 0),
	}
}

func (dp *dfsParam) dfsFinderLoop(currentState gluskov.State) {
	if dp.visited[currentState] {
		for i, state := range dp.path {
			if state == currentState {
				cycle := dp.path[i:]
				dp.cycles = append(dp.cycles, cycle)
				return
			}
		}
		return
	}

	dp.visited[currentState] = true
	dp.path = append(dp.path, currentState)

	transitions := dp.machine.Transitions[currentState]
	for _, nextStates := range transitions {
		for _, nextState := range nextStates {
			dp.dfsFinderLoop(nextState)
		}
	}

	dp.path = dp.path[:len(dp.path)-1]
}

type StateLoopToString map[gluskov.State][]LetterLoop

type LetterLoop struct {
	SecondState gluskov.State
	FinalState  gluskov.State
	AlpLoop     string
}

func TranslateLoops(s [][]gluskov.State, m *gluskov.Machine) StateLoopToString {
	sts := make(StateLoopToString)

	for counterLoops := 0; counterLoops < len(s); counterLoops++ {
		startCycle := s[counterLoops][0]
		if _, ok := sts[startCycle]; !ok {
			sts[startCycle] = make([]LetterLoop, 0)
		}
		sts[startCycle] = append(sts[startCycle], translateLoop(s[counterLoops], m))
	}

	return sts
}

func newLetterLoop(s []gluskov.State) LetterLoop {
	if len(s) > 1 {
		return LetterLoop{
			SecondState: s[1],
			FinalState:  s[len(s)-1],
		}
	}
	return LetterLoop{
		SecondState: s[0],
		FinalState:  s[len(s)-1],
	}
}

func translateLoop(s []gluskov.State, m *gluskov.Machine) LetterLoop {

	letterLoop := newLetterLoop(s)
	for i := 0; i < len(s)-1; i++ {
		current := s[i]
		next := s[i+1]
		letter, _ := m.GetRuneBetweenStates(current, next)
		letterLoop.AlpLoop += string(letter)
	}

	return letterLoop
}
