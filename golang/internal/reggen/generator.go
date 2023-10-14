package reggen

import (
	"fmt"
	"math/rand"
)

const (
	maxAlphabetNumber = 51
)

type Regexes struct {
	rgp        RegexGeneratorParams
	CountRegex int
}

type RegexGeneratorParams struct {
	alphabetSize int
	starHeight   int
	letterCount  int
}

func New(
	countRegex int,
	alphabetSize int,
	starHeight int,
	letterCount int,
) (*Regexes, error) {
	rgp, err := NewRGP(alphabetSize, starHeight, letterCount)
	if err != nil {
		return &Regexes{}, err
	}
	return &Regexes{
		rgp:        *rgp,
		CountRegex: countRegex,
	}, nil
}

func NewRGP(
	alphabetSize int,
	starHeight int,
	letterCount int,
) (*RegexGeneratorParams, error) {
	if alphabetSize > maxAlphabetNumber {
		return &RegexGeneratorParams{},
			fmt.Errorf("max alphabet size is %d, get: %d\n", maxAlphabetNumber, alphabetSize)
	}
	return &RegexGeneratorParams{
		alphabetSize: alphabetSize,
		starHeight:   starHeight,
		letterCount:  letterCount,
	}, nil
}

func (r *Regexes) Generate() []string {

	regexes := make([]string, r.CountRegex)

	for i := 0; i < r.CountRegex; i++ {
		regexes[i] = r.rgp.Generate()
	}

	return regexes
}

func (rgp *RegexGeneratorParams) Generate() string {

	regex := rgp.generateRegexRecursive(rgp.letterCount, rgp.alphabetSize)

	if len(regex) == 0 {
		return rgp.Generate()
	}
	return regex
}

func (rgp *RegexGeneratorParams) generateRegexRecursive(maxLength int, maxStarHeight int) string {
	if maxLength <= 0 || maxStarHeight <= 0 {
		return ""
	}

	operation := rand.Intn(4)

	switch operation {
	case 0:
		left := rgp.generateRegexRecursive(maxLength/2, maxStarHeight)
		right := rgp.generateRegexRecursive(maxLength/2, maxStarHeight)
		return concat(left, right)
	case 1:
		left := rgp.generateRegexRecursive(maxLength-1, maxStarHeight)
		right := rgp.generateRegexRecursive(maxLength-1, maxStarHeight)
		return alternative(left, right)
	case 2:
		subExpr := rgp.generateRegexRecursive(maxLength-1, maxStarHeight-1)
		return starKellie(subExpr)
	default:
		letter := getLetter(rand.Intn(rgp.alphabetSize))
		return letter
	}
}

func concat(left, right string) string {
	return left + right
}

func alternative(left, right string) string {
	if len(left) > 0 && len(right) > 0 {
		return "(" + left + "|" + right + ")"
	}
	return ""
}

func starKellie(expr string) string {
	if len(expr) > 1 {
		return "(" + expr + ")" + "*"
	} else if len(expr) == 1 {
		return expr + "*"
	} else {
		return ""
	}
}

func getLetter(numLetter int) string {
	return string('a' + byte(numLetter))
}
