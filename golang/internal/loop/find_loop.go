package loop

import (
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/gluskov"
)

type dfsParam struct {
	machine *gluskov.Machine
	cycles  [][]loopItem
	visited map[gluskov.State]bool
	path    []loopItem
}

type loopItem struct {
	state   gluskov.State
	literal rune
}

func FindCycles(machine *gluskov.Machine) [][]loopItem {

	dfsp := newDfsParam(machine)
	dfsp.dfsFinderLoop(gluskov.State(0), ' ')

	return dfsp.cycles
}

func newDfsParam(machine *gluskov.Machine) *dfsParam {
	return &dfsParam{
		machine: machine,
		cycles:  make([][]loopItem, 0),
		visited: make(map[gluskov.State]bool),
		path:    make([]loopItem, 0),
	}
}

func (dp *dfsParam) dfsFinderLoop(currentState gluskov.State, incomingLiteral rune) {
	if dp.visited[currentState] {
		for i, state := range dp.path {
			if state.state == currentState {
				cycle := dp.path[i:]
				dp.cycles = append(dp.cycles, cycle)
				return
			}
		}
		return
	}

	dp.visited[currentState] = true
	dp.path = append(dp.path, loopItem{currentState, incomingLiteral})

	transitions := dp.machine.Transitions[currentState]
	for literal, nextStates := range transitions {
		for _, nextState := range nextStates {
			dp.dfsFinderLoop(nextState, literal)
		}
	}

	dp.path = dp.path[:len(dp.path)-1]
}

type StateLoopToString map[gluskov.State][]letterLoop

type letterLoop struct {
	SecondState gluskov.State
	FinalState  gluskov.State
	AlpLoop     string
}

func TranslateLoops(s [][]loopItem) StateLoopToString {
	sts := make(StateLoopToString)

	for counterLoops := 0; counterLoops < len(s); counterLoops++ {
		startCycle := s[counterLoops][0]
		if _, ok := sts[startCycle.state]; !ok {
			sts[startCycle.state] = make([]letterLoop, 0)
		}
		sts[startCycle.state] = append(sts[startCycle.state], translateLoop(s[counterLoops]))
	}

	return sts
}

func newLetterLoop(s []loopItem) letterLoop {
	if len(s) > 1 {
		return letterLoop{
			SecondState: s[1].state,
			FinalState:  s[len(s)-1].state,
		}
	}
	return letterLoop{
		SecondState: s[0].state,
		FinalState:  s[0].state,
	}
}

func translateLoop(s []loopItem) letterLoop {

	lp := newLetterLoop(s)
	for i := 0; i < len(s); i++ {
		lp.AlpLoop += string(s[i].literal)
	}

	return lp
}
