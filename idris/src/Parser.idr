module Parser

import Data.String
import Regex

public export
data Parser a = MkParser (List Char -> Maybe (List Char, a))

Functor Parser where
  map f (MkParser p) = MkParser (\arg => 
              do (input', res) <- p arg
                 Just (input', f res)
              )

Applicative Parser where
  pure val = MkParser (\arg => Just (arg, val))
  (<*>) (MkParser f_parser) (MkParser old) = MkParser (\arg => 
                do (input', fn) <- f_parser arg
                   (input'', res) <- old input'
                   Just (input'', fn res)
                )

Alternative Parser where
  empty = MkParser (\arg => Nothing)
  (MkParser left) <|> rightP = MkParser (\arg => 
           case left arg of
              (Just x) => Just x
              Nothing => let (MkParser right) = rightP in 
                             right arg
           )

public export
runParser : Parser a -> List Char -> Maybe (List Char, a)
runParser (MkParser f) cs = f cs    

charP : Char -> Parser Char
charP c = MkParser (\arg => case arg of
                                 [] => Nothing
                                 (x :: xs) => if x == c 
                                                 then Just (xs, x)
                                                 else Nothing
                  )

alternatives: Parser Regex

group : Parser Regex
group = toGroup <$> (charP '(' *> alternatives <* charP ')')
  where
    toGroup : Regex -> Regex
    toGroup (Alt x y z) = Group (Alt x y z) z
    toGroup (Concat x y z) = Group (Concat x y z) z
    toGroup (Group x y) = Group x y 
    toGroup (Star x) = Star x 
    toGroup (Chr c) = Group (Chr c) False 
    

spanP : (Char -> Bool) -> Parser (List Char)
spanP f = MkParser (\arg => 
          let (first, second) = span f arg in 
              Just (second, first)
          )

isAlphaP : Parser Regex
isAlphaP = MkParser (\arg => 
           case arg of
                [] => Nothing
                (x :: xs) => 
                    if (isAlphaNum x) 
                       then Just (xs, Chr x)
                       else Nothing
           )


||| парсит блоки вида (...)*** или .*
||| возвращает либо саму группу, либо символ, либо звезду
star : Parser Regex
star = MkParser (\arg => 
       let starConsumer = spanP (\x => x == '*') in
       let groupOpt = (runParser group arg) in 
           case groupOpt of
                Just (input', groupRes) => 
                   let starsOpt = runParser starConsumer input' in
                       case starsOpt of
                          Just (afterStars, stars) => 
                              if null stars 
                                 then Just(afterStars, groupRes)
                                 else Just(afterStars, Star groupRes)
                          Nothing => Just(input', groupRes) 
                Nothing => 
                  do (input', chr) <- runParser isAlphaP arg 
                     let starsOpt = runParser starConsumer input'
                     case starsOpt of 
                          Just (afterStars, stars) => 
                              if null stars
                                 then Just (afterStars, chr)
                                 else Just (afterStars, Star chr)
                          Nothing => Just (input', chr)
               )

many : Parser Regex -> Parser (List Regex)
many (MkParser parse) = MkParser (\arg => Just (parseRec arg))
  where
    parseRec: List Char -> (List Char, List Regex)
    parseRec input = case parse input of
                          Nothing => (input, [])
                          Just (input', res) => let (remainder, xs) = parseRec input' in
                                                    (remainder, res :: xs)


concat : Parser Regex
concat = MkParser (\arg => 
         do (input', res) <- runParser (many star) arg
            case res of
                 (x :: xs) => Just (input', toConcat x xs)
                 [] => Nothing
         )


alternatives = MkParser (\arg => 
        do (input', res) <- runParser concat arg
           (let (remainder, xs) = parseRec input' in 
                Just (remainder, makeAlt res xs)
            )
        )
  where
    parseRec: List Char -> (List Char, List Regex)
    parseRec input = case runParser ((charP '|') *> concat) input of
                          Nothing => (input, [])
                          Just (input', res) => let (remainder, xs) = parseRec input' in
                                                    (remainder, res :: xs)



public export
regex : Parser ERegex
regex = (distribute . ACINormalize . ssnf) <$> alternatives

