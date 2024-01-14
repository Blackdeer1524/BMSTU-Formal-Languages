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
         Nothing => "error: input is NOT a regex\n"
         Just (x, y) => do
           if null x 
              then (toString y) ++ "\n"
              else "error: couldn't parse regex\n"


main : IO ()
main = repl "" transform 
