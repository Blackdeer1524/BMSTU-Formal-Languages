package parsing

import (
	"fmt"
	"strings"
	"testing"
)

func printSet(m map[rune]struct{}) string {
	var builder strings.Builder
	builder.WriteRune('{')
	for item := range m {
		builder.WriteString(fmt.Sprintf("%c, ", item))
	}
	builder.WriteRune('}')
	return builder.String()
}

func TestEps(t *testing.T) {
	info := GrammarInfo{
		Terms:       map[rune]struct{}{},
		Productions: map[rune][]string{},
	}
	info.Terms['a'] = struct{}{}
	info.Terms['$'] = struct{}{}
	info.Terms[EPSILON] = struct{}{}

	info.Productions['S'] = []string{"aA$"}
	info.Productions['A'] = []string{"B", "C"}
	info.Productions['B'] = []string{"Ba", string(EPSILON)}
	info.Productions['C'] = []string{"a"}

	res := getEpsInfo(info)

	if entry, ok := res['S']; !ok {
		t.Fatal("Variable S not found")
	} else {
		if entry {
			t.Fatal("expected false")
		}
	}

	if entry, ok := res['B']; !ok {
		t.Fatal("Variable B not found")
	} else {
		if !entry {
			t.Fatal("expected true")
		}
	}

	if entry, ok := res['C']; !ok {
		t.Fatal("Variable C not found")
	} else {
		if entry {
			t.Fatal("expected false")
		}
	}

	if entry, ok := res['A']; !ok {
		t.Fatal("Variable A not found")
	} else {
		if !entry {
			t.Fatal("expected true")
		}
	}
}

func testFirstHelper(t *testing.T, res map[rune]map[rune]struct{}, v rune, n int, exp []rune) {
	if len(res[v]) != n {
		t.Errorf("Expeceted %d, found %d", n, len(res[v]))
	}

	for _, e := range exp {
		if _, ok := res[v][e]; !ok {
			t.Errorf("Expected '%c' to be in First(%c)", e, v)
		}
	}
}

func TestFirst(t *testing.T) {
	info := GrammarInfo{
		Terms:       map[rune]struct{}{},
		Productions: map[rune][]string{},
	}
	info.Terms['a'] = struct{}{}
	info.Terms['b'] = struct{}{}
	info.Terms['$'] = struct{}{}
	info.Terms[EPSILON] = struct{}{}

	info.Productions['S'] = []string{"aA$"}
	info.Productions['A'] = []string{"B", "C"}
	info.Productions['B'] = []string{"Ba", string(EPSILON)}
	info.Productions['C'] = []string{"b"}

	epsInfo := getEpsInfo(info)
	res := getFirstInfo(info, epsInfo)

	testFirstHelper(t, res, 'S', 1, []rune{'a'})
	testFirstHelper(t, res, 'A', 3, []rune{'a', 'b', EPSILON})
	testFirstHelper(t, res, 'C', 1, []rune{'b'})
}

func TestFirstHard(t *testing.T) {
	info := GrammarInfo{
		Terms:       map[rune]struct{}{},
		Productions: map[rune][]string{},
	}
	info.Terms['+'] = struct{}{}
	info.Terms['*'] = struct{}{}
	info.Terms['i'] = struct{}{}
	info.Terms['a'] = struct{}{}
	info.Terms['('] = struct{}{}
	info.Terms[')'] = struct{}{}
	info.Terms['$'] = struct{}{}
	info.Terms[EPSILON] = struct{}{}

	// E' = Q; T' = P
	info.Productions['S'] = []string{"E$"}
	info.Productions['E'] = []string{"TQ"}
	info.Productions['Q'] = []string{"+TQ", string(EPSILON)}
	info.Productions['T'] = []string{"FP"}
	info.Productions['P'] = []string{"*FP", string(EPSILON)}
	info.Productions['F'] = []string{"i", "(E)"}

	epsInfo := getEpsInfo(info)
	res := getFirstInfo(info, epsInfo)

	testFirstHelper(t, res, 'E', 2, []rune{'i', '('})
	testFirstHelper(t, res, 'Q', 2, []rune{'+', EPSILON})
	testFirstHelper(t, res, 'T', 2, []rune{'i', '('})
	testFirstHelper(t, res, 'P', 2, []rune{'*', EPSILON})
	testFirstHelper(t, res, 'F', 2, []rune{'i', '('})
}

func testFollowHelper(t *testing.T, res map[rune]map[rune]struct{}, v rune, n int, exp []rune) {
	if len(res[v]) != n {
		t.Errorf("[%c] Expeceted %d, found %d", v, n, len(res[v]))
	}

	for _, e := range exp {
		if _, ok := res[v][e]; !ok {
			t.Errorf("Expected '%c' to be in Follow(%c) = %s", e, v, printSet(res[v]))
		}
	}
}

func TestFollow(t *testing.T) {
	info := GrammarInfo{
		Terms:       map[rune]struct{}{},
		Productions: map[rune][]string{},
	}
	info.Terms['+'] = struct{}{}
	info.Terms['*'] = struct{}{}
	info.Terms['i'] = struct{}{}
	info.Terms['a'] = struct{}{}
	info.Terms['('] = struct{}{}
	info.Terms[')'] = struct{}{}
	info.Terms['$'] = struct{}{}
	info.Terms[EPSILON] = struct{}{}

	// E' = Q; T' = P
	info.Productions['S'] = []string{"E$"}
	info.Productions['E'] = []string{"TQ"}
	info.Productions['Q'] = []string{"+TQ", string(EPSILON)}
	info.Productions['T'] = []string{"FP"}
	info.Productions['P'] = []string{"*FP", string(EPSILON)}
	info.Productions['F'] = []string{"i", "(E)"}

	epsInfo := getEpsInfo(info)
	firstInfo := getFirstInfo(info, epsInfo)
	res := getFollowInfo(info, epsInfo, firstInfo)

	testFollowHelper(t, res, 'E', 2, []rune{'$', ')'})
	testFollowHelper(t, res, 'Q', 2, []rune{'$', ')'})
	testFollowHelper(t, res, 'T', 3, []rune{'+', '$', ')'})
	testFollowHelper(t, res, 'P', 3, []rune{'+', '$', ')'})
	testFollowHelper(t, res, 'F', 4, []rune{'+', '*', '$', ')'})
}
