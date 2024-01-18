import Decidable.Equality 
import Data.Vect
import Data.Either
import Data.String


mutual
  data TypeEmpty = TEmpty

  data TypeNone = TNone

  data TypeChar = TChar Char

  data TypeAlt = TAlt Regex Regex 

  data TypeConcat = TConcat SimpleRegex Regex 

  data TypeStar = TStar Regex 

  data SimpleRegex : Type where
    SEmpty : TypeEmpty -> SimpleRegex
    SNone : TypeNone -> SimpleRegex
    SChr : TypeChar -> SimpleRegex
    SStar : TypeStar -> SimpleRegex
    SAlt : TypeAlt -> SimpleRegex 

  data Regex : Type where
    Unwrap : SimpleRegex -> Regex
    Concat : SimpleRegex -> Regex -> Regex


Empty : Regex
Empty = Unwrap (SEmpty TEmpty)

None : Regex
None = Unwrap (SNone TNone)

Chr : Char -> Regex
Chr c = Unwrap (SChr (TChar c))

Star : Regex -> Regex
Star x = Unwrap (SStar (TStar x))

Alt : Regex -> Regex -> Regex
Alt x y = Unwrap (SAlt (TAlt x y))

isNull : Regex -> Bool

runRegex : Regex -> List Char -> Maybe (List (List Char))

flatten : List (List a) -> List a
flatten [] = []
flatten (x :: xs) = x ++ flatten xs

runPlus : Regex -> List Char -> List (List Char)
runPlus x cs = case (runRegex x cs) of
                    (Just y) => y ++ (flatten $ map (runPlus x) y)
                    Nothing => []

runRegex (Unwrap (SEmpty TEmpty)) x = Just [x]
runRegex (Unwrap (SNone TNone)) cs = Nothing
runRegex (Unwrap (SChr (TChar c))) [] = Nothing
runRegex (Unwrap (SChr (TChar c))) (x :: xs) = 
  if x == c 
     then Just [xs]
     else Nothing
runRegex (Unwrap (SStar (TStar x))) cs = Just ([cs] ++ (runPlus x cs))
runRegex (Unwrap (SAlt (TAlt x y))) cs = let left = (runRegex x cs) in
                                         let right = (runRegex y cs) in 
                                             case left of
                                                  (Just z) => case right of
                                                                   (Just w) => Just (z ++ w)
                                                                   Nothing => Just z
                                                  Nothing => right

runRegex (Concat x y) cs = 
  let left = (runRegex (Unwrap x) cs) in 
      case left of
           (Just z) => case flatten $ filterNothings $ map (runRegex y) z of
                            (w :: xs) => Just (w :: xs)
                            [] => Nothing
           Nothing => Nothing
  where
    filterNothings : List (Maybe a) -> List a
    filterNothings [] = []
    filterNothings (x :: xs) = case x of
                                  Nothing => []
                                  (Just y) => y :: (filterNothings xs)




