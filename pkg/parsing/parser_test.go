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

	w0 := "i+i+i+i"
	w1 := "i*i+i+i"

	table := BuildTable(info)
	p := newLL1Parser(table, info.Terms)

	T0 := p.BuildTree(w0)

	T1 := Incremental(w0, T0, w1, info, true)
	T1IncDebug := T1.Debug()

	T1Full := p.BuildTree(w1)
	T1FullDebug := T1Full.Debug()

	if T1FullDebug != T1IncDebug {
		t.Fatalf("incremental & full trees don't match: %s | %s", T1IncDebug, T1FullDebug)
	}
	
}
