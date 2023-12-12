package parsing

import "testing"

func TestParser(t *testing.T) {
	info := GrammarInfo{
		Terms: map[string]struct{}{
			"+":     {},
			"*":     {},
			"i":     {},
			"a":     {},
			"(":     {},
			")":     {},
			"$":     {},
			EPSILON: {},
		},
		Productions: map[string][][]string{
			"S": {{"E", "$"}},
			"E": {{"T", "Q"}},
			"Q": {{"+", "T", "Q"}, {EPSILON}},
			"T": {{"F", "P"}},
			"P": {{"*", "F", "P"}, {EPSILON}},
			"F": {{"i"}, {"(", "E", ")"}},
		},
	}

	// E' = Q; T' = P
	info.Productions["S"] = [][]string{{"E", "$"}}
	info.Productions["E"] = [][]string{{"T", "Q"}}
	info.Productions["Q"] = [][]string{{"+", "T", "Q"}, {EPSILON}}
	info.Productions["T"] = [][]string{{"F", "P"}}
	info.Productions["P"] = [][]string{{"*", "F", "P"}, {EPSILON}}
	info.Productions["F"] = [][]string{{"i"}, {"(", "E", ")"}}

	foo("i+i+i+i", "i*i+i+i", info, true)
}
