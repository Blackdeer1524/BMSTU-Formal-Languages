package parser

import (
	"fmt"
	"os"
	"os/exec"
	"regexp/syntax"
	"strconv"
	"strings"
)

func ParseRegex(regex string) (*syntax.Regexp, error) {
	return syntax.Parse(regex, 0)
}

const (
	dotFileName = "lab2/syntax_tree.dot"
	pngFileName = "lab2/syntax_tree.png"
)

var nodeCounter int

func ParseRegexInDot(regex string) error {
	parseTree, pErr := syntax.Parse(regex, 0)
	if pErr != nil {
		return fmt.Errorf("Ошибка разбора регулярного выражения:%w\n", pErr)
	}

	dotFile, cErr := os.Create(dotFileName)
	if cErr != nil {
		return fmt.Errorf("Ошибка создания DOT-файла: %w\n", cErr)
	}
	defer func() {
		_ = dotFile.Close()
	}()

	_, _ = dotFile.WriteString("digraph SyntaxTree {\n")

	nodeCounter = 0

	mainNodeName := "node" + strconv.Itoa(nodeCounter)
	nodeCounter++
	_, _ = dotFile.WriteString(mainNodeName + " [label=\"Regex\"];\n")

	writeSyntaxTree(dotFile, mainNodeName, parseTree)

	_, _ = dotFile.WriteString("}\n")

	cmd := exec.Command("dot", "-Tpng", dotFileName, "-o", pngFileName)
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("Ошибка визуализации DOT-файла: %w\n", err)
	}

	fmt.Println("Создан файл с синтаксическим деревом:", pngFileName)
	return nil
}

func writeSyntaxTree(dotFile *os.File, parentName string, node *syntax.Regexp) {
	nodeName := "node" + strconv.Itoa(nodeCounter)
	nodeCounter++

	label := node.Op.String()
	if node.Op == syntax.OpLiteral {
		label += " (" + escapeForLabel(node.String()) + ")"
	}

	_, _ = dotFile.WriteString(nodeName + " [label=\"" + label + "\"];\n")
	_, _ = dotFile.WriteString(parentName + " -> " + nodeName + ";\n")

	if node.Sub != nil {
		for _, subNode := range node.Sub {
			writeSyntaxTree(dotFile, nodeName, subNode)
		}
	}
}

func escapeForLabel(s string) string {
	s = strings.ReplaceAll(s, "\\", "\\\\")
	s = strings.ReplaceAll(s, "\"", "\\\"")
	return s
}
