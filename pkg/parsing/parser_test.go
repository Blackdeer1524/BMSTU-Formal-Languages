package parsing

import "testing"

func grammarInfoFixture() GrammarInfo {
	info := GrammarInfo{
		Terms: map[string]struct{}{
			"+":     {},
			"*":     {},
			"i":     {},
			"a":     {},
			"(":     {},
			")":     {},
			EOS:     {},
			EPSILON: {},
		},
		Productions: map[string][][]string{
			"S": {{"E", EOS}},
			"E": {{"T", "Q"}},
			"Q": {{"+", "T", "Q"}, {EPSILON}},
			"T": {{"F", "P"}},
			"P": {{"*", "F", "P"}, {EPSILON}},
			"F": {{"i"}, {"(", "E", ")"}},
		},
	}

	// E' = Q; T' = P
	info.Productions["S"] = [][]string{{"E", EOS}}
	info.Productions["E"] = [][]string{{"T", "Q"}}
	info.Productions["Q"] = [][]string{{"+", "T", "Q"}, {EPSILON}}
	info.Productions["T"] = [][]string{{"F", "P"}}
	info.Productions["P"] = [][]string{{"*", "F", "P"}, {EPSILON}}
	info.Productions["F"] = [][]string{{"i"}, {"(", "E", ")"}}
	return info
}

func TestNotInLang(t *testing.T) {
	panicCaught := false
	defer func() {
		if err := recover(); err != nil {
			panicCaught = true
		}
	}()

	info := grammarInfoFixture()
	table := BuildTable(info)
	p := NewLL1Parser(table, info.Terms)

	p.BuildTree("i+i+")

	if !panicCaught {
		t.Fatalf("expected panic")
	}
}

func TestSimple(t *testing.T) {
	info := grammarInfoFixture()
	w0 := "i+i+i+i"
	w1 := "i*i+i+i"

	table := BuildTable(info)
	p := NewLL1Parser(table, info.Terms)

	T0 := p.BuildTree(w0)

	T1 := Incremental(w0, T0, w1, info, true)
	T1IncDebug := T1.Debug()

	T1Full := p.BuildTree(w1)
	T1FullDebug := T1Full.Debug()

	if T1FullDebug != T1IncDebug {
		t.Fatalf("incremental & full trees don't match: %s | %s", T1IncDebug, T1FullDebug)
	}
}

func TestEmptyPrefix(t *testing.T) {
	info := grammarInfoFixture()

	w0 := "(i+i)+i+i"
	w1 := "i+i+i+i"

	table := BuildTable(info)
	p := NewLL1Parser(table, info.Terms)

	T0 := p.BuildTree(w0)

	T1 := Incremental(w0, T0, w1, info, true)
	T1IncDebug := T1.Debug()

	T1Full := p.BuildTree(w1)
	T1FullDebug := T1Full.Debug()

	if T1FullDebug != T1IncDebug {
		t.Fatalf("incremental & full trees don't match: %s | %s", T1IncDebug, T1FullDebug)
	}
}

func TestEmptySuffixW0(t *testing.T) {
	info := grammarInfoFixture()

	w0 := "i+i"
	w1 := "i+i+i+i"

	table := BuildTable(info)
	p := NewLL1Parser(table, info.Terms)

	T0 := p.BuildTree(w0)

	T1 := Incremental(w0, T0, w1, info, true)
	T1IncDebug := T1.Debug()

	T1Full := p.BuildTree(w1)
	T1FullDebug := T1Full.Debug()

	if T1FullDebug != T1IncDebug {
		t.Fatalf("incremental & full trees don't match: %s | %s", T1IncDebug, T1FullDebug)
	}
}

func TestEmptySuffixW1(t *testing.T) {
	info := grammarInfoFixture()

	w0 := "i+i+i+i"
	w1 := "i+i"

	table := BuildTable(info)
	p := NewLL1Parser(table, info.Terms)

	T0 := p.BuildTree(w0)

	T1 := Incremental(w0, T0, w1, info, true)
	T1IncDebug := T1.Debug()

	T1Full := p.BuildTree(w1)
	T1FullDebug := T1Full.Debug()

	if T1FullDebug != T1IncDebug {
		t.Fatalf("incremental & full trees don't match: %s | %s", T1IncDebug, T1FullDebug)
	}
}
