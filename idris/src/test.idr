import Decidable.Equality 
import Data.Vect
import Data.Either
import Data.String


mutual
  data TypeEmpty = TEmpty

  data TypeNone = TNone

  data TypeChar = TChar Char

  data TypeAlt = TAlt Regex Regex 

  data TypeConcat = TConcat Regex Regex 

  data TypeStar = TStar Regex 

  data Regex : Type where
    Empty  : TypeEmpty -> Regex 
    None   : TypeNone -> Regex 
    Chr    : TypeChar -> Regex 
    Star   : TypeStar -> Regex 
    Alt    : TypeAlt -> Regex 
    Concat : TypeConcat -> Regex 

isNull : Regex -> Bool

runRegex : Regex -> List Char -> Maybe (List (List Char))

flatten : List (List a) -> List a
flatten [] = []
flatten (x :: xs) = x ++ flatten xs

runPlus : Regex -> List Char -> List (List Char)
runPlus x cs = case (runRegex x cs) of
                    (Just y) => y ++ (flatten $ map (runPlus x) y)
                    Nothing => []

runRegex (Empty TEmpty) x = Just [x]
runRegex (None TNone) cs = Nothing
runRegex (Chr (TChar c)) [] = Nothing
runRegex (Chr (TChar c)) (x :: xs) = 
  if x == c 
     then Just [xs]
     else Nothing
runRegex (Star (TStar x)) cs = Just ([cs] ++ (runPlus x cs))
runRegex (Alt (TAlt x y)) cs = let left = (runRegex x cs) in
                                         let right = (runRegex y cs) in 
                                             case left of
                                                  (Just z) => case right of
                                                                   (Just w) => Just (z ++ w)
                                                                   Nothing => Just z
                                                  Nothing => right

runRegex (Concat (TConcat x y)) cs = 
  let left = (runRegex x cs) in 
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




