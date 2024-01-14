module Main

import Data.Vect
import Data.String
import System.REPL

import Parser
import Regex

run : String -> IO ()
run str = 
  do 
    let opt = runParser regex (unpack str)
    case opt of
         Nothing => printLn "Nothing"
         Just (x, y) => do
           printLn ("Remainder: \"" ++ (pack x) ++ "\"")
           printERegex (cast y)


transform : String -> String
transform str = 
    let opt = runParser regex (unpack (trim str)) in
    case opt of
         Nothing => "error: input is NOT a regex"
         Just (x, y) => do
           if null x 
              then (toString y)
              else "error: couldn't parse regex"


splitSep : String -> List String
splitSep str = pack <$> (helper [] (unpack str))
  where
    helper: (acc : List Char) -> (List Char) -> List (List Char)
    helper [] [] = []
    helper (x :: xs) [] = [reverse (x :: xs)]
    helper acc (x :: xs) = if x == '\n'
                              then if length acc > 0 
                                      then reverse acc :: (helper [] xs)
                                      else (helper [] xs) 
                              else helper (x :: acc) xs 

inputProcess : String -> String
inputProcess str = let x = transform <$> (splitSep str) in 
                       trim (foldl (\acc, elem => acc ++ "\n" ++ elem) "" x) ++ "\n"



main : IO ()
main = repl "" inputProcess 
