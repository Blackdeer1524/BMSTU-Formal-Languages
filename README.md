Выполнил: Слесарев Данила, ИУ9-51Б

**Вариант 1**

# Компиляция

```bash
cargo build --release
```

Путь к скомпилированной программе: `./target/release/lab1`

Формулы переписывания принимаются через `stdin`.

Результат генерации выдаётся через `stdout`.

# Использование

* Генерация файла со спецификацией:

```bash
cat input.txt | ./target/release/lab1 > result.txt 
```

* Прогон с z3:

```bash
cat input.txt | ./target/release/lab1 | z3 -in 
```

