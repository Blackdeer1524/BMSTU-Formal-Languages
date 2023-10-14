package bencmark

import (
	"fmt"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/reggen"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/wordgen"
)

// Start TODO: надо получать путь до собранного бинарника
func Start(reg *reggen.Regexes, rustBinaryPath string) error {

	// TODO: подключаюсь к расту

	regexes := reg.Generate()
	words, err := wordgen.GenerateWordsForRegexes(regexes, 15, 5)
	if err != nil {
		return fmt.Errorf("in bench start %w", err)
	}

	// TODO: кидаю в раст регулярки и получаю ответ

	// TODO: запускаю бенчмарк
	fmt.Println("words")
	for i, w := range words {
		fmt.Println(i+1, ":", w)
	}

	return nil
}
