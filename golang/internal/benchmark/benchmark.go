package benchmark

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"runtime"
	"strings"
	"time"

	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/reggen"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/wordgen"
)

var (
	_, b, _, _       = runtime.Caller(0)
	basepath         = filepath.Dir(b)
	pythonScriptPath = basepath + "/regular_compression.py"
)

func EquivalenceCheck(reg *reggen.Regexes, rustBinaryPath string, countWords, maxDumpSize int) error {
	words, err := prepareEnvironment(reg, rustBinaryPath, countWords, maxDumpSize)
	if err != nil {
		return err
	}

	compareRegexWithWords(words)

	return nil
}

func compareRegexWithWords(rwws []wordgen.RegexesWithWords) {

	for _, rww := range rwws {
		fmt.Printf("compare expected: %s regular with actual: %s\n", rww.RegexBefore, rww.RegexAfter)
		runWords(rww)
	}
}

func runWords(rww wordgen.RegexesWithWords) {
	for _, word := range rww.Words {
		if !equalMatched("^"+rww.RegexBefore+"$", "^"+rww.RegexAfter+"$", word) {
			fmt.Printf("Don`t equal in word: %s\n", word)
		} else {
			fmt.Printf("OK in: %s\n", word)
		}
	}
}

func equalMatched(p1, p2, word string) bool {
	beforeMatched, _ := regexp.MatchString(p1, word)
	afterMatched, _ := regexp.MatchString(p2, word)
	return beforeMatched == afterMatched
}

func Start(reg *reggen.Regexes, rustBinaryPath string, countWords, maxDumpSize int) error {

	words, err := prepareEnvironment(reg, rustBinaryPath, countWords, maxDumpSize)
	if err != nil {
		return err
	}

	words, err = wordgen.GenerateWordsForBenchmarkRegexes(words)
	if err != nil {
		return err
	}

	pErr := runBenchmarksInPython(words)
	if pErr != nil {
		return fmt.Errorf("failed at bench start python comparassion %w", pErr)
	}
	return nil
}

func prepareEnvironment(
	reg *reggen.Regexes, rustBinaryPath string, countWords, maxDumpSize int,
) ([]wordgen.RegexesWithWords, error) {

	regexes := reg.Generate()

	fmt.Println("started generating...")

	fmt.Println("regexes", regexes)

	words, gErr := wordgen.GenerateWordsForRegexes(regexes, countWords, maxDumpSize)
	if gErr != nil {
		return nil, fmt.Errorf("failed in bench start generate words %w", gErr)
	}

	cErr := conversionRegularExpression(words, rustBinaryPath)
	if cErr != nil {
		return nil, fmt.Errorf("failed at bench start conversion %w", cErr)
	}

	return words, nil
}

func conversionRegularExpression(rww []wordgen.RegexesWithWords, rustBinaryPath string) error {
	cmd := exec.Command(rustBinaryPath)

	var stdin, stdout bytes.Buffer

	cmd.Stdin = &stdin
	cmd.Stdout = &stdout

	for _, regexWords := range rww {
		stdin.WriteString(regexWords.RegexBefore + "\n")
	}

	rErr := cmd.Run()
	if rErr != nil {
		return fmt.Errorf("failed to run simplifier: %w", rErr)
	}

	output := strings.TrimSuffix(stdout.String(), "\n")
	outputStrings := strings.Split(output, "\n")

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
	durBefore, okBefore, beforeErr := runPythonScriptForOneRegex(
		wordsWithRegex.RegexBefore,
		wordsWithRegex.Words,
	)
	if beforeErr != nil {
		return fmt.Errorf("failed to run before regexp %w", beforeErr)
	}

	durAfter, okAfter, afterErr := runPythonScriptForOneRegex(
		wordsWithRegex.RegexAfter,
		wordsWithRegex.Words,
	)
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
