package bencmark

import (
	"bytes"
	"fmt"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/reggen"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/wordgen"
	"os"
	"os/exec"
	"strings"
)

func Start(reg *reggen.Regexes, rustBinaryPath string) error {

	regexes := reg.Generate()
	words, gErr := wordgen.GenerateWordsForRegexes(regexes, 15, 5)
	if gErr != nil {
		return fmt.Errorf("in bench start generate words %w", gErr)
	}

	cErr := conversionRegularExpression(words, rustBinaryPath)
	if cErr != nil {
		return fmt.Errorf("in bench start conversion %w", cErr)
	}

	// TODO: запускаю бенчмарк
	fmt.Println("words")
	for i, w := range words {
		fmt.Println(i+1, ":", w)
	}

	return nil
}

func conversionRegularExpression(rww []wordgen.RegexesWithWords, rustBinaryPath string) error {

	fmt.Printf("рабочая директория: ")
	fmt.Println(os.Getwd())

	cmd := exec.Command(rustBinaryPath)

	var stdin, stdout bytes.Buffer

	cmd.Stdin = &stdin
	cmd.Stdout = &stdout

	for _, regexWords := range rww {
		stdin.WriteString(regexWords.RegexBefore + "\n")
	}

	rErr := cmd.Run()
	if rErr != nil {
		return fmt.Errorf("failed to run exec rust binary %w", rErr)
	}

	output := strings.TrimSuffix(stdout.String(), "\n\n")

	outputStrings := strings.Split(output, "\n\n")

	for i, newRegex := range outputStrings {
		rww[i].RegexAfter = newRegex
	}

	return nil
}
