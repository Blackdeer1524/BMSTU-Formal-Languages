from dataclasses import dataclass, field
from enum import Enum
from typing import Literal, Optional


class State(Enum):
    NOT_SEEN = 1
    HAS_TO_TERMINATE = 2
    TERMINATES = 3

    def __str__(self) -> str:
        match self.value:
            case 1:
                return "NOT SEEN"
            case 2:
                return "SEEN"
            case 3:
                return "TERMINATES"
        raise NotImplementedError(f"{self.value}")


NONTERMINALS_T = Literal["A", "B", "S"]
NONTERMINALS = ["S", "A", "B"]
TERMINALS = ["a", "b", "Ɛ"]


@dataclass(frozen=True, repr=False)
class TerminationData:
    rule_parent: Optional[NONTERMINALS_T] = None

    S: State = field(default=State.NOT_SEEN)
    A: State = field(default=State.NOT_SEEN)
    B: State = field(default=State.NOT_SEEN)

    def is_final(self) -> bool:
        return (
            self.S == State.TERMINATES
            and self.rule_parent is None
            and self.A != State.HAS_TO_TERMINATE
            and self.B != State.HAS_TO_TERMINATE
        )

    def __repr__(self) -> str:
        res = ""
        if self.rule_parent is None:
            res = "[START]\n"
        else:
            res = f"[{self.rule_parent}->]\n"

        res += f"S: {self.S}\nA: {self.A}\nB: {self.B}"
        return res


@dataclass(frozen=True)
class Edge:
    target: TerminationData
    regex: str


TRANSITIONS: dict[TerminationData, set[Edge]] = {
    TerminationData(S=s, A=a, B=b, rule_parent=parent): set()  # type: ignore
    for parent in NONTERMINALS + [None]
    for s in (State.HAS_TO_TERMINATE, State.NOT_SEEN, State.TERMINATES)
    for a in (State.HAS_TO_TERMINATE, State.NOT_SEEN, State.TERMINATES)
    for b in (State.HAS_TO_TERMINATE, State.NOT_SEEN, State.TERMINATES)
}

VISITED = {
    TerminationData(S=s, A=a, B=b, rule_parent=parent): False  # type: ignore
    for parent in NONTERMINALS + [None]
    for s in (State.HAS_TO_TERMINATE, State.NOT_SEEN, State.TERMINATES)
    for a in (State.HAS_TO_TERMINATE, State.NOT_SEEN, State.TERMINATES)
    for b in (State.HAS_TO_TERMINATE, State.NOT_SEEN, State.TERMINATES)
}


def visit_after_arrow(after_arrow_state: TerminationData):
    if VISITED[after_arrow_state]:
        return
    VISITED[after_arrow_state] = True

    for nonterm_name in NONTERMINALS:
        inits = {v: after_arrow_state.__getattribute__(v) for v in NONTERMINALS}
        inits[nonterm_name] = (
            State.HAS_TO_TERMINATE
            if after_arrow_state.__getattribute__(nonterm_name) != State.TERMINATES
            else State.TERMINATES
        )
        for c in TERMINALS[:-1]:
            next_xT = TerminationData(**inits)
            TRANSITIONS[after_arrow_state].add(
                Edge(
                    target=next_xT,
                    regex=f"{c}{nonterm_name}$",
                ),
            )
            visit_rule_start(next_xT)


def visit_rule_start(current: TerminationData):
    if VISITED[current]:
        return
    VISITED[current] = True

    for nonterm_name in NONTERMINALS:
        inits = {v: current.__getattribute__(v) for v in NONTERMINALS}
        inits[nonterm_name] = (
            State.TERMINATES
            if current.__getattribute__(nonterm_name) == State.TERMINATES
            else State.HAS_TO_TERMINATE
        )

        after_T_arrow = TerminationData(rule_parent=nonterm_name, **inits)  # type: ignore
        TRANSITIONS[current].add(
            Edge(
                target=after_T_arrow,
                regex=f"{nonterm_name}->",
            )
        )
        visit_after_arrow(after_T_arrow)
        for c in TERMINALS:
            inner_inits = {v: after_T_arrow.__getattribute__(v) for v in NONTERMINALS}
            inner_inits[nonterm_name] = State.TERMINATES

            after_T_rhs = TerminationData(
                **inner_inits,
            )
            TRANSITIONS[after_T_arrow].add(
                Edge(
                    target=after_T_rhs,
                    regex=f"{c}$",
                ),
            )
            visit_rule_start(after_T_rhs)


if __name__ == "__main__":
    visit_rule_start(TerminationData())
    print("digraph {")
    print("    rankdir=LR")
    print(
        "    node [shape = doublecircle]; {};".format(
            "".join(
                [
                    '"' + str(item) + '"'
                    for item in TRANSITIONS.keys()
                    if item.is_final()
                ]
            )
        )
    )
    print("    node [shape = circle];")
    for start, edges in TRANSITIONS.items():
        for edge in edges:
            print(
                f'    "{str(start)}" -> "{str(edge.target)}" [label = "{edge.regex}"];'
            )
    print("}")
