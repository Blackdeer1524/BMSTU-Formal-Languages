package benchmark

import (
	"bytes"
	"fmt"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/reggen"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/wordgen"
	"os"
	"os/exec"
	"strings"
	"time"
)

const pythonScriptPath = "golang/internal/benchmark/regular_compression.py"

func Start(reg *reggen.Regexes, rustBinaryPath string, countWords, maxDumpSize int) error {

	regexes := reg.Generate()

	fmt.Println("start generate")

	words, gErr := wordgen.GenerateWordsForRegexes(regexes, countWords, maxDumpSize)
	if gErr != nil {
		return fmt.Errorf("failed in bench start generate words %w", gErr)
	}

	fmt.Println("end generate")

	cErr := conversionRegularExpression(words, rustBinaryPath)
	if cErr != nil {
		return fmt.Errorf("failed in bench start conversion %w", cErr)
	}

	pErr := runBenchmarksInPython(words)
	if pErr != nil {
		return fmt.Errorf("failed in bench start python comparassion %w", cErr)
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

func runBenchmarksInPython(wordsWithRegexes []wordgen.RegexesWithWords) error {

	for _, wordsWithRegex := range wordsWithRegexes {
		err := runPythonScriptForPairRegexes(wordsWithRegex)
		if err != nil {
			return fmt.Errorf("failed to run pair script: %w", err)
		}
	}

	return nil
}

func runPythonScriptForPairRegexes(wordsWithRegex wordgen.RegexesWithWords) error {

	durBefore, okBefore, beforeErr := runPythonScriptForOneRegex(wordsWithRegex.RegexBefore, wordsWithRegex.Words)
	if beforeErr != nil {
		return fmt.Errorf("failed to run before regexp %w", beforeErr)
	}

	durAfter, okAfter, afterErr := runPythonScriptForOneRegex(wordsWithRegex.RegexAfter, wordsWithRegex.Words)
	if afterErr != nil {
		return fmt.Errorf("failed to run after regexp %w", afterErr)
	}

	fmt.Printf(
		"\tto before: regex: %s, status: %s, duration: %s\n",
		wordsWithRegex.RegexBefore, okBefore, durBefore,
	)
	fmt.Printf(
		"\tto after: regex: %s, status: %s, duration: %s\n",
		wordsWithRegex.RegexAfter, okAfter, durAfter,
	)

	fmt.Println("_______________________")

	return nil
}

func runPythonScriptForOneRegex(regexp string, words []string) (*time.Duration, string, error) {
	cmd := exec.Command("python3", pythonScriptPath, fmt.Sprintf("\"%s\"", regexp))

	var stdin, stdout bytes.Buffer

	cmd.Stdin = &stdin
	cmd.Stdout = &stdout
	cmd.Stderr = os.Stderr

	startTime := time.Now()

	if err := cmd.Start(); err != nil {
		return nil, "", fmt.Errorf("failed to start python script %w", err)
	}

	for _, word := range words {
		stdin.WriteString(word + "\n")
	}

	go func() {
		<-time.After(time.Duration(len(words)) * time.Second)
		if cmd.Process != nil {
			err := cmd.Process.Kill()
			if err != nil {
				fmt.Println("failed to kill process")
			}
		}
	}()

	if err := cmd.Wait(); err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return nil, "", fmt.Errorf("failed to wait script %s", string(exitErr.Stderr))
		} else {
			return nil, "", fmt.Errorf("failed to wait script %s", string(exitErr.Stderr))
		}
	}

	elapsedTime := time.Since(startTime)

	output := strings.TrimSuffix(stdout.String(), "\n")

	return &elapsedTime, output, nil
}
