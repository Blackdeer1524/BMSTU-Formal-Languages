package parsing

import (
	"fmt"
	"strings"
	"testing"
)

func printSet(m map[string]struct{}) string {
	var builder strings.Builder
	builder.WriteRune('{')
	for item := range m {
		builder.WriteString(item)
	}
	builder.WriteRune('}')
	return builder.String()
}

func TestEps(t *testing.T) {
	info := GrammarInfo{
		Terms:       map[string]struct{}{},
		Productions: map[string][][]string{},
	}
	info.Terms["a"] = struct{}{}
	info.Terms[EOS] = struct{}{}
	info.Terms[EPSILON] = struct{}{}

	info.Productions["S"] = [][]string{{"a", "A", EOS}}
	info.Productions["A"] = [][]string{{"B"}, {"C"}}
	info.Productions["B"] = [][]string{{"B", "a"}, {EPSILON}}
	info.Productions["C"] = [][]string{{"a"}}

	res := getEpsInfo(info)

	if entry, ok := res["S"]; !ok {
		t.Fatal("Variable S not found")
	} else {
		if entry {
			t.Fatal("expected false")
		}
	}

	if entry, ok := res["B"]; !ok {
		t.Fatal("Variable B not found")
	} else {
		if !entry {
			t.Fatal("expected true")
		}
	}

	if entry, ok := res["C"]; !ok {
		t.Fatal("Variable C not found")
	} else {
		if entry {
			t.Fatal("expected false")
		}
	}

	if entry, ok := res["A"]; !ok {
		t.Fatal("Variable A not found")
	} else {
		if !entry {
			t.Fatal("expected true")
		}
	}
}

func testFirstHelper(res map[string]map[string]struct{}, v string, n int, exp []string) error {
	var message strings.Builder
	if len(res[v]) != n {
		message.WriteString(fmt.Sprintf("Expeceted %d, found %d; ", n, len(res[v])))
	}

	for _, e := range exp {
		if _, ok := res[v][e]; !ok {
			message.WriteString(fmt.Sprintf("Expected '%s' to be in First(%s) = %v; ", e, v, res[v]))
		}
	}
	if message.Len() > 0 {
		return fmt.Errorf("%s", message.String())
	}

	return nil
}

func TestFirst(t *testing.T) {
	info := GrammarInfo{
		Terms:       map[string]struct{}{},
		Productions: map[string][][]string{},
	}
	info.Terms["a"] = struct{}{}
	info.Terms["b"] = struct{}{}
	info.Terms[EOS] = struct{}{}
	info.Terms[EPSILON] = struct{}{}

	info.Productions["S"] = [][]string{{"a", "A", EOS}}
	info.Productions["A"] = [][]string{{"B"}, {"C"}}
	info.Productions["B"] = [][]string{{"B", "a"}, {EPSILON}}
	info.Productions["C"] = [][]string{{"b"}}

	epsInfo := getEpsInfo(info)
	res := getFirstInfo(info, epsInfo)

	{
		err := testFirstHelper(res, "S", 1, []string{"a"})
		if err != nil {
			t.Error(err.Error())
		}
	}
	{
		err := testFirstHelper(res, "A", 3, []string{"a", "b", EPSILON})
		if err != nil {
			t.Error(err.Error())
		}
	}
	{
		err := testFirstHelper(res, "C", 1, []string{"b"})
		if err != nil {
			t.Error(err.Error())
		}
	}
}

func TestFirstHard(t *testing.T) {
	info := GrammarInfo{
		Terms:       map[string]struct{}{},
		Productions: map[string][][]string{},
	}
	info.Terms["+"] = struct{}{}
	info.Terms["*"] = struct{}{}
	info.Terms["i"] = struct{}{}
	info.Terms["a"] = struct{}{}
	info.Terms["("] = struct{}{}
	info.Terms[")"] = struct{}{}
	info.Terms[EOS] = struct{}{}
	info.Terms[EPSILON] = struct{}{}

	// E' = Q; T' = P
	info.Productions["S"] = [][]string{{"E", EOS}}
	info.Productions["E"] = [][]string{{"T", "Q"}}
	info.Productions["Q"] = [][]string{{"+", "T", "Q"}, {EPSILON}}
	info.Productions["T"] = [][]string{{"F", "P"}}
	info.Productions["P"] = [][]string{{"*", "F", "P"}, {EPSILON}}
	info.Productions["F"] = [][]string{{"i"}, {"(", "E", ")"}}

	epsInfo := getEpsInfo(info)
	res := getFirstInfo(info, epsInfo)

	{
		err := testFirstHelper(res, "E", 2, []string{"i", "("})
		if err != nil {
			t.Error(err.Error())
		}
	}
	{
		err := testFirstHelper(res, "Q", 2, []string{"+", EPSILON})
		if err != nil {
			t.Error(err.Error())
		}
	}
	{
		err := testFirstHelper(res, "T", 2, []string{"i", "("})
		if err != nil {
			t.Error(err.Error())
		}
	}
	{
		err := testFirstHelper(res, "P", 2, []string{"*", EPSILON})
		if err != nil {
			t.Error(err.Error())
		}
	}
	{
		err := testFirstHelper(res, "F", 2, []string{"i", "("})
		if err != nil {
			t.Error(err.Error())
		}
	}
}

func testFollowHelper(res map[string]map[string]struct{}, v string, n int, exp []string) error {
	var message strings.Builder
	for _, e := range exp {
		if _, ok := res[v][e]; !ok {
			message.WriteString(fmt.Sprintf("Expected '%s' to be in Follow(%s) = %s; ", e, v, printSet(res[v])))
		}
	}
	if len(res[v]) != n {
		message.WriteString(fmt.Sprintf("[%s] length mismatch: expected %d found %d (%s); ", v, n, len(res[v]), printSet(res[v])))
	}

	if message.Len() > 0 {
		return fmt.Errorf("%s", message.String())
	}

	return nil
}

func TestFollow(t *testing.T) {
	info := GrammarInfo{
		Terms:       map[string]struct{}{},
		Productions: map[string][][]string{},
	}
	info.Terms["+"] = struct{}{}
	info.Terms["*"] = struct{}{}
	info.Terms["i"] = struct{}{}
	info.Terms["a"] = struct{}{}
	info.Terms["("] = struct{}{}
	info.Terms[")"] = struct{}{}
	info.Terms[EOS] = struct{}{}
	info.Terms[EPSILON] = struct{}{}

	// E' = Q; T' = P
	info.Productions["S"] = [][]string{{"E", EOS}}
	info.Productions["E"] = [][]string{{"T", "Q"}}
	info.Productions["Q"] = [][]string{{"+", "T", "Q"}, {EPSILON}}
	info.Productions["T"] = [][]string{{"F", "P"}}
	info.Productions["P"] = [][]string{{"*", "F", "P"}, {EPSILON}}
	info.Productions["F"] = [][]string{{"i"}, {"(", "E", ")"}}

	epsInfo := map[string]bool{
		"S": false,
		"E": false,
		"Q": true,
		"T": false,
		"P": true,
		"F": false,
	}
	firstInfo := map[string]map[string]struct{}{
		"S": {"i": struct{}{}, "(": struct{}{}},
		"E": {"i": struct{}{}, "(": struct{}{}},
		"Q": {"+": struct{}{}, EPSILON: struct{}{}},
		"T": {"i": struct{}{}, "(": struct{}{}},
		"P": {"*": struct{}{}, EPSILON: struct{}{}},
		"F": {"i": struct{}{}, "(": struct{}{}},
	}
	res := getFollowInfo(info, epsInfo, firstInfo)

	{
		err := testFollowHelper(res, "E", 2, []string{EOS, ")"})
		if err != nil {
			t.Error(err.Error())
		}
	}
	{
		err := testFollowHelper(res, "Q", 2, []string{EOS, ")"})
		if err != nil {
			t.Error(err.Error())
		}
	}
	{
		err := testFollowHelper(res, "T", 3, []string{"+", EOS, ")"})
		if err != nil {
			t.Error(err.Error())
		}
	}
	{
		err := testFollowHelper(res, "P", 3, []string{"+", EOS, ")"})
		if err != nil {
			t.Error(err.Error())
		}
	}
	{
		err := testFollowHelper(res, "F", 4, []string{"+", "*", EOS, ")"})
		if err != nil {
			t.Error(err.Error())
		}
	}
}

func testTableHelper(table map[rune]map[rune]string, v rune, t rune, exp string) error {
	if table[v][t] != exp {
		return fmt.Errorf("expected %s, but %s found", exp, table[v][t])
	}
	return nil
}

func TestTable(t *testing.T) {
	info := GrammarInfo{
		Terms:       map[string]struct{}{},
		Productions: map[string][][]string{},
	}
	info.Terms["+"] = struct{}{}
	info.Terms["*"] = struct{}{}
	info.Terms["i"] = struct{}{}
	info.Terms["a"] = struct{}{}
	info.Terms["("] = struct{}{}
	info.Terms[")"] = struct{}{}
	info.Terms[EOS] = struct{}{}
	info.Terms[EPSILON] = struct{}{}

	// E' = Q; T' = P
	info.Productions["S"] = [][]string{{"E", EOS}}
	info.Productions["E"] = [][]string{{"T", "Q"}}
	info.Productions["Q"] = [][]string{{"+", "T", "Q"}, {EPSILON}}
	info.Productions["T"] = [][]string{{"F", "P"}}
	info.Productions["P"] = [][]string{{"*", "F", "P"}, {EPSILON}}
	info.Productions["F"] = [][]string{{"i"}, {"(", "E", ")"}}

	res := BuildTable(info)

	// E' = Q; T' = P
	exp := map[string]map[string][]string{
		"E": {"i": {"T", "Q"}, "(": {"T", "Q"}},
		"Q": {"+": {"+", "T", "Q"}, ")": {"ε"}, EOS: {"ε"}},
		"T": {"i": {"F", "P"}, "(": {"F", "P"}},
		"P": {"+": {"ε"}, "*": {"*", "F", "P"}, ")": {"ε"}, EOS: {"ε"}},
		"F": {"i": {"i"}, "(": {"(", "E", ")"}},
	}
	for v, row := range exp {
		for term, entry := range row {
			left := res[v][term]
			if len(left) != len(entry) {
				t.Errorf("[%s][%s] missmatch: expected %s, got %s", v, term, entry, res[v][term])
			}

			for i := 0; i < len(left); i++ {
				a := left[i]
				b := entry[i]
				if a != b {
					t.Errorf("[%s][%s] missmatch: expected %s, got %s", v, term, entry, res[v][term])
					break
				}
			}
		}
	}
}
