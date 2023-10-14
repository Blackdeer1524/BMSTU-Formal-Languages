import re
import sys


def my_python_function(r_pattern):
    all_is_ok = True
    for line in sys.stdin:
        word = line.strip()
        # Слово полностью соответствует регулярному выражению
        if re.search(f"^{r_pattern}$", word):
            all_is_ok = False
    return all_is_ok


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Не хватает аргументов")
        sys.exit(1)

    pattern = sys.argv[1]

    all_ok = my_python_function(pattern)
    if all_ok:
        print("OK")
    else:
        print("NOT OK")
