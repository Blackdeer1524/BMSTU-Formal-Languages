import re
import sys


def regex_tester(r_pattern: re.Pattern) -> bool:
    for line in sys.stdin:
        word = line[:-1]
        if re.fullmatch(r_pattern, word):
            return False
    return True


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Не хватает аргументов")
        sys.exit(1)

    raw_pattern = sys.argv[1]
    pattern = re.compile(raw_pattern)

    all_ok = regex_tester(pattern)
    if all_ok:
        print("OK")
    else:
        print("NOT OK")
