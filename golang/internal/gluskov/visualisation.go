package gluskov

import (
	"fmt"
	"os"
	"os/exec"
)

const (
	dotFileName = "lab2/machine.dot"
	pngFileName = "lab2/machine.png"
)

func (m *Machine) GetDotMachine() error {
	dot := m.toDOT()
	dotFile, cErr := os.Create(dotFileName)
	if cErr != nil {
		return fmt.Errorf("Ошибка создания DOT-файла: %w\n", cErr)
	}
	defer func() {
		_ = dotFile.Close()
	}()

	_, wErr := dotFile.WriteString(dot)
	if wErr != nil {
		return wErr
	}
	cmd := exec.Command("dot", "-Tpng", dotFileName, "-o", pngFileName)
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("Ошибка визуализации DOT-файла: %w\n", err)
	}
	fmt.Println("Создан файл с синтаксическим деревом:", pngFileName)
	return nil
}

func (m *Machine) toDOT() string {
	dot := "digraph Automaton {\n"
	dot += "\trankdir=LR;\n"

	dot += fmt.Sprintf("\t%d -> %d;\n", -1, m.StartState)
	dot += fmt.Sprintf("\t%d [shape=\"point\"];\n", -1)

	for _, state := range m.FinalStates {
		dot += fmt.Sprintf("\t%d [shape=\"doublecircle\"];\n", state)
	}

	for fromState, transitions := range m.Transitions {
		for symbol, toStates := range transitions {
			for _, toState := range toStates {
				dot += fmt.Sprintf("\t%d -> %d [label=\"%s\"];\n", fromState, toState, string(symbol))
			}
		}
	}

	dot += "}"
	return dot
}
