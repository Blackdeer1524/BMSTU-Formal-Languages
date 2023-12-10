package utils

// ExtractFirstRune extracts first rune from a given string
func ExtractFirstRune(s string) rune {
	var first rune
	for _, c := range s {
		first = c
		break
	}
	return first
}

// ExtractPair extracts first two runes from a given string
func ExtractPair(str string) (f rune, s rune) {
	for i, c := range str {
		if i == 0 {
			f = c
		} else {
			s = c
		}
	}
	return
}

// MergeInPlace merges two sets together
func MergeInPlace(dst map[rune]struct{}, src map[rune]struct{}) {
	for key := range src {
		dst[key] = struct{}{}
	}
}
