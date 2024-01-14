module Main

import Data.Vect
import Data.String

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


main : IO ()
main = putStrLn "Hello from Idris2!"
