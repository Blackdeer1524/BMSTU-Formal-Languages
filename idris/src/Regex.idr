module Regex

import Data.String

public export
data Regex = Alt Regex Regex Bool
           | Concat Regex Regex Bool
           | Group Regex Bool
           | Star Regex
           | Chr Char

Eq Regex where
  (Alt x y z) == (Alt w v s) = (x == w) && (y == v)
  (Chr c) == (Chr d) = c == d
  (Concat x y z) == (Concat w v s) = (x == w) && (y == v)
  (Group x y) == (Group z w) = x == z
  (Star x) == (Star y) = x == y
  _ == _ = False

RegToString : Regex -> String
RegToString (Alt x y z) = "(" ++ (RegToString x) ++ "|" ++ (RegToString y) ++ ")"
RegToString (Concat x y z) = (RegToString x) ++ (RegToString y)
RegToString (Group x y) = "(" ++ (RegToString x) ++ ")"
RegToString (Star x) = "(" ++ (RegToString x) ++ ")*"
RegToString (Chr c) = pack [c]

Ord Regex where
  left < right = (RegToString left) < (RegToString right) 

prepend: Nat -> Char -> String -> String
prepend 0 c str = str
prepend (S k) c str = (pack [c]) ++ (prepend k c str)

bool2string : Bool -> String
bool2string x = if x
                   then "True"
                   else "False"

printRegex : Regex -> Nat -> IO ()
printRegex (Alt x y z) k = 
  do 
     (printLn (prepend (k * 3) ' ' "Alt [" ++ bool2string z ++ "]" ))
     (printRegex x (S k))
     (printRegex y (S k))

printRegex (Concat x y z) k = 
  do 
     (printLn (prepend (k * 3) ' ' "Concat [" ++ bool2string z ++ "]" ))
     (printRegex x (k + 1))
     (printRegex y (k + 1))

printRegex (Group x y) k = 
  do 
    (printLn (prepend (k * 3) ' ' "Group [" ++ bool2string y ++ "]" )) 
    (printRegex x (S k))

printRegex (Star x) k = 
  do 
    (printLn (prepend (k * 3) ' ' "Star"))
    (printRegex x (S k))

printRegex (Chr c) k =
  do (printLn (prepend (k * 3) ' ' "Char " ++ pack [c]))


public export 
data ERegex = EAlt ERegex ERegex Bool
             | EConcat ERegex ERegex Bool
             | EGroup ERegex Bool
             | EStar ERegex
             | EChr Char
             | Eps

Eq ERegex where
  (EAlt x y z) == (EAlt w v s) = (x == w) && (y == v)
  (EChr c) == (EChr d) = c == d
  (EConcat x y z) == (EConcat w v s) = (x == w) && (y == v)
  (EGroup x y) == (EGroup z w) = x == z
  (EStar x) == (EStar y) = x == y
  Eps == Eps = True
  _ == _ = False


toString : ERegex -> String
toString (EAlt x y z) = "(" ++ (toString x) ++ "|" ++ (toString y) ++ ")"
toString (EConcat x y z) = (toString x) ++ (toString y)
toString (EGroup x y) = "(" ++ (toString x) ++ ")"
toString (EStar x) = "(" ++ (toString x) ++ ")*"
toString (EChr c) = pack [c]
toString Eps = ""

Ord ERegex where
  left < right = (toString left) < (toString right) 

Cast (Regex) (ERegex) where
  cast (Alt x y z) = EAlt (cast x) (cast y) z
  cast (Concat x y z) = (EConcat (cast x) (cast y) z)
  cast (Group x y) = (EGroup (cast x) y)
  cast (Star x) = (EStar (cast x))
  cast (Chr c) = (EChr c)

printERegexHelper : ERegex -> Nat -> IO ()
printERegexHelper (EAlt x y z) k = 
  do 
     (printLn (prepend (k * 3) ' ' "Alt [" ++ bool2string z ++ "]" ))
     (printERegexHelper x (S k))
     (printERegexHelper y (S k))

printERegexHelper Eps k = 
  do (printLn (prepend (k * 3) ' ' "Eps "))

printERegexHelper (EConcat x y z) k = 
  do 
     (printLn (prepend (k * 3) ' ' "Concat [" ++ bool2string z ++ "]" ))
     (printERegexHelper x (k + 1))
     (printERegexHelper y (k + 1))

printERegexHelper (EGroup x y) k = 
  do 
    (printLn (prepend (k * 3) ' ' "Group [" ++ bool2string y ++ "]" )) 
    (printERegexHelper x (S k))

printERegexHelper (EStar x) k = 
  do 
    (printLn (prepend (k * 3) ' ' "Star"))
    (printERegexHelper x (S k))

printERegexHelper (EChr c) k =
  do (printLn (prepend (k * 3) ' ' "Char " ++ pack [c]))


public export
printERegex : ERegex -> IO ()
printERegex x = 
  do (printLn (toString x))
     (printERegexHelper x 0)


export
toConcat : (acc: Regex) -> List Regex -> Regex
toConcat acc [] = acc
toConcat (Alt w v s)    ((Alt x y z) :: xs)    = toConcat (Concat (Alt w v s)    (Alt x y z) (s && z)) xs
toConcat (Alt w v s)    ((Concat x y z) :: xs) = toConcat (Concat (Alt w v s)    (Concat x y z) (s && z)) xs
toConcat (Alt x y z)    ((Chr c) :: xs)        = toConcat (Concat (Alt x y z)    (Chr c) False) xs
toConcat (Alt y z w)    ((Star x) :: xs)       = toConcat (Concat (Alt y z w)    (Star x) w) xs
toConcat (Alt z w v)    ((Group x y) :: xs)    = toConcat (Concat (Alt z w v)    (Group x y) (v && y)) xs
toConcat (Chr c)        ((Alt x y z) :: xs)    = toConcat (Concat (Chr c)        (Alt x y z) False) xs
toConcat (Chr c)        ((Concat x y z) :: xs) = toConcat (Concat (Chr c)        (Concat x y z) False) xs
toConcat (Chr c)        ((Group x y) :: xs)    = toConcat (Concat (Chr c)        (Group x y) False) xs
toConcat (Chr c)        ((Star x) :: xs)       = toConcat (Concat (Chr c)        (Star x) False) xs
toConcat (Chr d)        ((Chr c) :: xs)        = toConcat (Concat (Chr d)        (Chr c) False) xs
toConcat (Concat w v s) ((Alt x y z) :: xs)    = toConcat (Concat (Concat w v s) (Alt x y z) (s && z)) xs
toConcat (Concat w v s) ((Concat x y z) :: xs) = toConcat (Concat (Concat w v s) (Concat x y z) (s && z)) xs
toConcat (Concat x y z) ((Chr c) :: xs)        = toConcat (Concat (Concat x y z) (Chr c) False) xs
toConcat (Concat y z w) ((Star x) :: xs)       = toConcat (Concat (Concat y z w) (Star x) w) xs
toConcat (Concat z w v) ((Group x y) :: xs)    = toConcat (Concat (Concat z w v) (Group x y) (v && y)) xs
toConcat (Group w v)    ((Alt x y z) :: xs)    = toConcat (Concat (Group w v)    (Alt x y z) (v && z)) xs
toConcat (Group w v)    ((Concat x y z) :: xs) = toConcat (Concat (Group w v)    (Concat x y z) (v && z)) xs
toConcat (Group x y)    ((Chr c) :: xs)        = toConcat (Concat (Group x y)    (Chr c) False) xs
toConcat (Group y z)    ((Star x) :: xs)       = toConcat (Concat (Group y z)    (Star x) z) xs
toConcat (Group z w)    ((Group x y) :: xs)    = toConcat (Concat (Group z w)    (Group x y) (w && y)) xs
toConcat (Star w)       ((Alt x y z) :: xs)    = toConcat (Concat (Star w)       (Alt x y z) z) xs
toConcat (Star w)       ((Concat x y z) :: xs) = toConcat (Concat (Star w)       (Concat x y z) z) xs
toConcat (Star x)       ((Chr c) :: xs)        = toConcat (Concat (Star x)       (Chr c) False) xs
toConcat (Star y)       ((Star x) :: xs)       = toConcat (Concat (Star y)       (Star x) True) xs
toConcat (Star z)       ((Group x y) :: xs)    = toConcat (Concat (Star z)       (Group x y) y) xs

export
makeAlt: (acc: Regex) -> List Regex -> Regex 
makeAlt acc [] = acc
makeAlt (Alt w v s)    ((Alt x y z) :: xs)    = makeAlt (Alt (Alt w v s)    (Alt x y z) (s || z)) xs
makeAlt (Alt w v s)    ((Concat x y z) :: xs) = makeAlt (Alt (Alt w v s)    (Concat x y z) (s || z)) xs
makeAlt (Alt x y z)    ((Chr c) :: xs)        = makeAlt (Alt (Alt x y z)    (Chr c) z) xs
makeAlt (Alt y z w)    ((Star x) :: xs)       = makeAlt (Alt (Alt y z w)    (Star x) True) xs
makeAlt (Alt z w v)    ((Group x y) :: xs)    = makeAlt (Alt (Alt z w v)    (Group x y) (v || y)) xs
makeAlt (Chr c)        ((Alt x y z) :: xs)    = makeAlt (Alt (Chr c)        (Alt x y z) z) xs
makeAlt (Chr c)        ((Concat x y z) :: xs) = makeAlt (Alt (Chr c)        (Concat x y z) z) xs
makeAlt (Chr c)        ((Group x y) :: xs)    = makeAlt (Alt (Chr c)        (Group x y) y) xs
makeAlt (Chr c)        ((Star x) :: xs)       = makeAlt (Alt (Chr c)        (Star x) True) xs
makeAlt (Chr d)        ((Chr c) :: xs)        = makeAlt (Alt (Chr d)        (Chr c) False) xs
makeAlt (Concat w v s) ((Alt x y z) :: xs)    = makeAlt (Alt (Concat w v s) (Alt x y z) (s || z)) xs
makeAlt (Concat w v s) ((Concat x y z) :: xs) = makeAlt (Alt (Concat w v s) (Concat x y z) (s || z)) xs
makeAlt (Concat x y z) ((Chr c) :: xs)        = makeAlt (Alt (Concat x y z) (Chr c) z) xs
makeAlt (Concat y z w) ((Star x) :: xs)       = makeAlt (Alt (Concat y z w) (Star x) True) xs
makeAlt (Concat z w v) ((Group x y) :: xs)    = makeAlt (Alt (Concat z w v) (Group x y) (v || y)) xs
makeAlt (Group w v)    ((Alt x y z) :: xs)    = makeAlt (Alt (Group w v)    (Alt x y z) (v || z)) xs
makeAlt (Group w v)    ((Concat x y z) :: xs) = makeAlt (Alt (Group w v)    (Concat x y z) (v || z)) xs
makeAlt (Group x y)    ((Chr c) :: xs)        = makeAlt (Alt (Group x y)    (Chr c) y) xs
makeAlt (Group y z)    ((Star x) :: xs)       = makeAlt (Alt (Group y z)    (Star x) z) xs
makeAlt (Group z w)    ((Group x y) :: xs)    = makeAlt (Alt (Group z w)    (Group x y) (w || y)) xs
makeAlt (Star w)       ((Alt x y z) :: xs)    = makeAlt (Alt (Star w)       (Alt x y z) True) xs
makeAlt (Star w)       ((Concat x y z) :: xs) = makeAlt (Alt (Star w)       (Concat x y z) True) xs
makeAlt (Star x)       ((Chr c) :: xs)        = makeAlt (Alt (Star x)       (Chr c) True) xs
makeAlt (Star y)       ((Star x) :: xs)       = makeAlt (Alt (Star y)       (Star x) True) xs
makeAlt (Star z)       ((Group x y) :: xs)    = makeAlt (Alt (Star z)       (Group x y) True) xs

ssnfHelper : ERegex -> ERegex

ss : ERegex -> ERegex
ss (EGroup x y) = ss x
ss (EStar x) = ss x
ss (EChr c) = EChr c
ss Eps = Eps 
ss (EAlt x y z) = (EAlt (ss x) (ss y) z)
ss (EConcat x y False) = (EConcat (ssnfHelper x) (ssnfHelper y) False)
ss (EConcat x y True) = (EAlt (ss x) (ss y) True)

ssnfHelper (EAlt x y z) = (EAlt (ssnfHelper x) (ssnfHelper y) z) 
ssnfHelper (EConcat x y z) = (EConcat (ssnfHelper x) (ssnfHelper y) z)
ssnfHelper (EGroup x y)  = (EGroup (ssnfHelper x) y)  -- тут точно остается инвариантность на y
ssnfHelper (EStar x)  = EStar (ss x)
ssnfHelper (EChr c)   = EChr c
ssnfHelper (Eps)   = Eps

altToList : ERegex -> List ERegex
altToList (EAlt x y z) = (altToList x) ++ (altToList y)
altToList x = [x]

makeEAlt: (acc: ERegex) -> List ERegex -> ERegex 
makeEAlt acc [] = acc
makeEAlt (EAlt w v s)    ((EAlt x y z) :: xs)    = makeEAlt (EAlt (EAlt w v s)    (EAlt x y z) (s || z)) xs
makeEAlt (EAlt w v s)    ((EConcat x y z) :: xs) = makeEAlt (EAlt (EAlt w v s)    (EConcat x y z) (s || z)) xs
makeEAlt (EAlt x y z)    ((EChr c) :: xs)        = makeEAlt (EAlt (EAlt x y z)    (EChr c) z) xs
makeEAlt (EAlt y z w)    ((EStar x) :: xs)       = makeEAlt (EAlt (EAlt y z w)    (EStar x) True) xs
makeEAlt (EAlt z w v)    ((EGroup x y) :: xs)    = makeEAlt (EAlt (EAlt z w v)    (EGroup x y) (v || y)) xs
makeEAlt (EAlt z w v)    (Eps :: xs)             = makeEAlt (EAlt (EAlt z w v)    Eps True) xs 
makeEAlt (EChr c)        ((EAlt x y z) :: xs)    = makeEAlt (EAlt (EChr c)        (EAlt x y z) z) xs
makeEAlt (EChr c)        ((EConcat x y z) :: xs) = makeEAlt (EAlt (EChr c)        (EConcat x y z) z) xs
makeEAlt (EChr c)        ((EGroup x y) :: xs)    = makeEAlt (EAlt (EChr c)        (EGroup x y) y) xs
makeEAlt (EChr c)        ((EStar x) :: xs)       = makeEAlt (EAlt (EChr c)        (EStar x) True) xs
makeEAlt (EChr d)        ((EChr c) :: xs)        = makeEAlt (EAlt (EChr d)        (EChr c) False) xs
makeEAlt (EChr d)        (Eps :: xs)             = makeEAlt (EAlt (EChr d)        Eps True) xs
makeEAlt (EConcat w v s) ((EAlt x y z) :: xs)    = makeEAlt (EAlt (EConcat w v s) (EAlt x y z) (s || z)) xs
makeEAlt (EConcat w v s) ((EConcat x y z) :: xs) = makeEAlt (EAlt (EConcat w v s) (EConcat x y z) (s || z)) xs
makeEAlt (EConcat x y z) ((EChr c) :: xs)        = makeEAlt (EAlt (EConcat x y z) (EChr c) z) xs
makeEAlt (EConcat y z w) ((EStar x) :: xs)       = makeEAlt (EAlt (EConcat y z w) (EStar x) True) xs
makeEAlt (EConcat z w v) ((EGroup x y) :: xs)    = makeEAlt (EAlt (EConcat z w v) (EGroup x y) (v || y)) xs
makeEAlt (EConcat z w v) (Eps :: xs)             = makeEAlt (EAlt (EConcat z w v) Eps True) xs
makeEAlt (EGroup w v)    ((EAlt x y z) :: xs)    = makeEAlt (EAlt (EGroup w v)    (EAlt x y z) (v || z)) xs
makeEAlt (EGroup w v)    ((EConcat x y z) :: xs) = makeEAlt (EAlt (EGroup w v)    (EConcat x y z) (v || z)) xs
makeEAlt (EGroup x y)    ((EChr c) :: xs)        = makeEAlt (EAlt (EGroup x y)    (EChr c) y) xs
makeEAlt (EGroup y z)    ((EStar x) :: xs)       = makeEAlt (EAlt (EGroup y z)    (EStar x) True) xs
makeEAlt (EGroup z w)    ((EGroup x y) :: xs)    = makeEAlt (EAlt (EGroup z w)    (EGroup x y) (w || y)) xs
makeEAlt (EGroup z w)    (Eps :: xs)             = makeEAlt (EAlt (EGroup z w)    Eps True) xs
makeEAlt (EStar w)       ((EAlt x y z) :: xs)    = makeEAlt (EAlt (EStar w)       (EAlt x y z) True) xs
makeEAlt (EStar w)       ((EConcat x y z) :: xs) = makeEAlt (EAlt (EStar w)       (EConcat x y z) True) xs
makeEAlt (EStar x)       ((EChr c) :: xs)        = makeEAlt (EAlt (EStar x)       (EChr c) True) xs
makeEAlt (EStar y)       ((EStar x) :: xs)       = makeEAlt (EAlt (EStar y)       (EStar x) True) xs
makeEAlt (EStar z)       ((EGroup x y) :: xs)    = makeEAlt (EAlt (EStar z)       (EGroup x y) True) xs
makeEAlt (EStar z)       (Eps :: xs)             = makeEAlt (EAlt (EStar z)       Eps True) xs
makeEAlt Eps             ((EAlt x y z) :: xs)    = makeEAlt (EAlt Eps             (EAlt x y z) True) xs
makeEAlt Eps             ((EConcat x y z) :: xs) = makeEAlt (EAlt Eps             (EConcat x y z) True) xs 
makeEAlt Eps             ((EGroup x y) :: xs)    = makeEAlt (EAlt Eps             (EGroup x y) True) xs  
makeEAlt Eps             ((EStar x) :: xs)       = makeEAlt (EAlt Eps             (EStar x) True) xs  
makeEAlt Eps             ((EChr c) :: xs)        = makeEAlt (EAlt Eps             (EChr c) True) xs  
makeEAlt Eps             (Eps :: xs)             = makeEAlt (EAlt Eps             Eps True) xs  


makeEConcat: (acc: ERegex) -> List ERegex -> ERegex 
makeEConcat acc [] = acc
makeEConcat (EAlt w v s)    ((EAlt x y z) :: xs)    = makeEConcat (EConcat (EAlt w v s)    (EAlt x y z) (s || z)) xs
makeEConcat (EAlt w v s)    ((EConcat x y z) :: xs) = makeEConcat (EConcat (EAlt w v s)    (EConcat x y z) (s || z)) xs
makeEConcat (EAlt x y z)    ((EChr c) :: xs)        = makeEConcat (EConcat (EAlt x y z)    (EChr c) z) xs
makeEConcat (EAlt y z w)    ((EStar x) :: xs)       = makeEConcat (EConcat (EAlt y z w)    (EStar x) True) xs
makeEConcat (EAlt z w v)    ((EGroup x y) :: xs)    = makeEConcat (EConcat (EAlt z w v)    (EGroup x y) (v || y)) xs
makeEConcat (EAlt z w v)    (Eps :: xs)             = makeEConcat (EConcat (EAlt z w v)    Eps True) xs 
makeEConcat (EChr c)        ((EAlt x y z) :: xs)    = makeEConcat (EConcat (EChr c)        (EAlt x y z) z) xs
makeEConcat (EChr c)        ((EConcat x y z) :: xs) = makeEConcat (EConcat (EChr c)        (EConcat x y z) z) xs
makeEConcat (EChr c)        ((EGroup x y) :: xs)    = makeEConcat (EConcat (EChr c)        (EGroup x y) y) xs
makeEConcat (EChr c)        ((EStar x) :: xs)       = makeEConcat (EConcat (EChr c)        (EStar x) True) xs
makeEConcat (EChr d)        ((EChr c) :: xs)        = makeEConcat (EConcat (EChr d)        (EChr c) False) xs
makeEConcat (EChr d)        (Eps :: xs)             = makeEConcat (EConcat (EChr d)        Eps True) xs
makeEConcat (EConcat w v s) ((EAlt x y z) :: xs)    = makeEConcat (EConcat (EConcat w v s) (EAlt x y z) (s || z)) xs
makeEConcat (EConcat w v s) ((EConcat x y z) :: xs) = makeEConcat (EConcat (EConcat w v s) (EConcat x y z) (s || z)) xs
makeEConcat (EConcat x y z) ((EChr c) :: xs)        = makeEConcat (EConcat (EConcat x y z) (EChr c) z) xs
makeEConcat (EConcat y z w) ((EStar x) :: xs)       = makeEConcat (EConcat (EConcat y z w) (EStar x) True) xs
makeEConcat (EConcat z w v) ((EGroup x y) :: xs)    = makeEConcat (EConcat (EConcat z w v) (EGroup x y) (v || y)) xs
makeEConcat (EConcat z w v) (Eps :: xs)             = makeEConcat (EConcat (EConcat z w v) Eps True) xs
makeEConcat (EGroup w v)    ((EAlt x y z) :: xs)    = makeEConcat (EConcat (EGroup w v)    (EAlt x y z) (v || z)) xs
makeEConcat (EGroup w v)    ((EConcat x y z) :: xs) = makeEConcat (EConcat (EGroup w v)    (EConcat x y z) (v || z)) xs
makeEConcat (EGroup x y)    ((EChr c) :: xs)        = makeEConcat (EConcat (EGroup x y)    (EChr c) y) xs
makeEConcat (EGroup y z)    ((EStar x) :: xs)       = makeEConcat (EConcat (EGroup y z)    (EStar x) True) xs
makeEConcat (EGroup z w)    ((EGroup x y) :: xs)    = makeEConcat (EConcat (EGroup z w)    (EGroup x y) (w || y)) xs
makeEConcat (EGroup z w)    (Eps :: xs)             = makeEConcat (EConcat (EGroup z w)    Eps True) xs
makeEConcat (EStar w)       ((EAlt x y z) :: xs)    = makeEConcat (EConcat (EStar w)       (EAlt x y z) True) xs
makeEConcat (EStar w)       ((EConcat x y z) :: xs) = makeEConcat (EConcat (EStar w)       (EConcat x y z) True) xs
makeEConcat (EStar x)       ((EChr c) :: xs)        = makeEConcat (EConcat (EStar x)       (EChr c) True) xs
makeEConcat (EStar y)       ((EStar x) :: xs)       = makeEConcat (EConcat (EStar y)       (EStar x) True) xs
makeEConcat (EStar z)       ((EGroup x y) :: xs)    = makeEConcat (EConcat (EStar z)       (EGroup x y) True) xs
makeEConcat (EStar z)       (Eps :: xs)             = makeEConcat (EConcat (EStar z)       Eps True) xs
makeEConcat Eps             ((EAlt x y z) :: xs)    = makeEConcat (EConcat Eps             (EAlt x y z) True) xs
makeEConcat Eps             ((EConcat x y z) :: xs) = makeEConcat (EConcat Eps             (EConcat x y z) True) xs 
makeEConcat Eps             ((EGroup x y) :: xs)    = makeEConcat (EConcat Eps             (EGroup x y) True) xs  
makeEConcat Eps             ((EStar x) :: xs)       = makeEConcat (EConcat Eps             (EStar x) True) xs  
makeEConcat Eps             ((EChr c) :: xs)        = makeEConcat (EConcat Eps             (EChr c) True) xs  
makeEConcat Eps             (Eps :: xs)             = makeEConcat (EConcat Eps             Eps True) xs  

public export
ACINormalize : ERegex -> ERegex
ACINormalize (EAlt x y z) =  
  let altList = map ACINormalize (unique (sort (altToList (EAlt x y z)))) in
      case altList of
         (w :: xs) => (makeEAlt w xs)
         [] => Eps  -- ok
  where
      unique : List ERegex -> List ERegex  
      unique [] = []
      unique (w :: []) = [w]
      unique (w :: (v :: xs)) = if w == v 
                                   then unique (v :: xs)
                                   else w :: unique (v :: xs)

ACINormalize (EConcat x y z) = (EConcat (ACINormalize x) (ACINormalize y) z)
ACINormalize (EGroup x y) = (EGroup (ACINormalize x) y) 
ACINormalize (EStar x) = (EStar (ACINormalize x)) 
ACINormalize (EChr c) = (EChr c) 
ACINormalize Eps = Eps 

public export
ssnf : Regex -> ERegex
ssnf x = ssnfHelper (cast x)

-- forceEpsPrefix : ERegex -> ERegex
-- forceEpsPrefix (EConcat Eps w v) = (EConcat Eps w v)
-- forceEpsPrefix (EConcat x w v) = (EConcat (EConcat Eps x False) w v)
-- forceEpsPrefix x = x

public export
flattenConcat : ERegex -> List ERegex
flattenConcat (EConcat Eps w v) = flattenConcat w
flattenConcat (EConcat x Eps v) = flattenConcat x 
flattenConcat (EConcat x y v) = flattenConcat x ++ (flattenConcat y)
flattenConcat x = [x]

public export
getCommonPrefix: (pref : List ERegex) -> (left : List ERegex) -> (right : List ERegex) -> (List ERegex, List ERegex, List ERegex)
getCommonPrefix pref (w :: xs) (v :: ys) = 
   if w == v 
       then getCommonPrefix (w :: pref) xs ys
       else (reverse pref, (w :: xs), (v :: ys))
getCommonPrefix pref [] x = (reverse pref, [], x)
getCommonPrefix pref x [] = (reverse pref, x, []) 

public export
getCommonSuffix: (left : List ERegex) -> (right : List ERegex) -> (List ERegex, List ERegex, List ERegex)
getCommonSuffix left right = 
  let (suff, resLeft, resRight) = getCommonPrefix [] (reverse left) (reverse right) in
      (reverse suff, reverse resLeft, reverse resRight)


a = getCommonSuffix (flattenConcat (EConcat (EConcat (EChr 'a') (EChr 'b') False) (EChr 'c') False)) (flattenConcat (EConcat (EChr 'a') (EChr 'c') False)) 


public export
distribute : ERegex -> ERegex
distribute (EAlt x y z) = 
  let left =  distribute x in
  let right = distribute y in
      case (left, right) of
           ((EConcat ll lr t), (EConcat rl rr x1)) => 
               let leftConcatArgs  = flattenConcat left  in 
               let rightConcatArgs = flattenConcat right in 
                   let (pref, leftConcatArgs, rightConcatArgs) = getCommonPrefix [] leftConcatArgs rightConcatArgs in 
                   let (suff, leftConcatArgs, rightConcatArgs) = getCommonSuffix    leftConcatArgs rightConcatArgs in 
                        case (length leftConcatArgs > 0, length rightConcatArgs > 0) of
                             (False, False) => left
                             (False, True) => (EConcat 
                                                  (EConcat 
                                                      (makeEConcat Eps pref) 
                                                      (EAlt 
                                                          Eps 
                                                          (makeEConcat Eps rightConcatArgs) 
                                                          False) 
                                                       False)
                                                   (makeEConcat Eps suff)
                                                   False)
                             (True, False) =>(EConcat 
                                                  (EConcat 
                                                      (makeEConcat Eps pref) 
                                                      (EAlt 
                                                          (makeEConcat Eps leftConcatArgs) 
                                                          Eps 
                                                          False) 
                                                       False)
                                                   (makeEConcat Eps suff)
                                                   False) 
                             (True, True) => (EConcat 
                                                  (EConcat 
                                                      (makeEConcat Eps pref) 
                                                      (EAlt 
                                                          (makeEConcat Eps leftConcatArgs) 
                                                          (makeEConcat Eps rightConcatArgs) 
                                                          False) 
                                                       False)
                                                   (makeEConcat Eps suff)
                                                   False) 
           x => (EAlt left right z)

  where

distribute (EConcat x y z) = (EConcat (distribute x) (distribute y) z) 
distribute (EGroup x y) = (EGroup (distribute x) y) 
distribute (EStar x) =(EStar (distribute x)) 
distribute (EChr c) = (EConcat Eps (EChr c) False)
distribute Eps = Eps

