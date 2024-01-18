import Decidable.Equality 
import Data.Vect
import Data.Either
import Data.String


mutual
  data Alt : Type -> Type -> Type where
    MkAlt: (a: RegexTypes) -> (b: RegexTypes) -> Alt (deduce a) (deduce b)

  data Star : Type -> Type where
    MkStar : (a : RegexTypes) -> Star (deduce a)

  data Chr : Char -> Type where
    MkChr : (c: Char) -> Chr c

  -- data Concat : Type -> Type -> Type where
  --   MkConcat : (a: RegexTypes) -> (b: RegexTypes) -> Concat (deduce a) (deduce b)

  data Group : Type -> Type where
    MkGroup : (a: RegexTypes) -> Group (deduce a)

  data Eps = MkEps

  data None = MkNone


  data RegexTypes = WrapperAlt (Alt a b)
                  | WrapperChr (Chr a)
                  | WrapperStar (Star a)
                  -- | WrapperConcat (Concat a b)
                  | WrapperGroup (Group a)
                  | WrapperEps Eps
                  | WrapperNone None

  deduce : RegexTypes -> Type 
  deduce (WrapperStar (MkStar a)) = Star (deduce a)
  deduce (WrapperChr (MkChr a)) = Chr a
  deduce (WrapperAlt (MkAlt a b)) = Alt (deduce a) (deduce b)
  -- deduce (WrapperConcat (MkConcat a b)) = Concat (deduce a) (deduce b)
  deduce (WrapperGroup (MkGroup a)) = Group (deduce a)
  deduce (WrapperEps MkEps) = Eps
  deduce (WrapperNone MkNone) = None

isNull : RegexTypes -> Bool
isNull (WrapperStar a) = True
-- isNull (WrapperConcat (MkConcat a b)) = (isNull a) && (isNull b)
isNull (WrapperAlt (MkAlt a b)) = (isNull a) || (isNull b)
isNull (WrapperChr (MkChr c)) = False
isNull (WrapperGroup (MkGroup a)) = isNull a
isNull (WrapperEps MkEps) = True
isNull (WrapperNone MkNone) = False

ss : RegexTypes -> Type
ssnf : RegexTypes -> Type

ssnf (WrapperAlt (MkAlt a b)) = Alt (ssnf a) (ssnf b)
ssnf (WrapperChr (MkChr a)) = Chr a
ssnf (WrapperStar (MkStar a)) = Star (ss a)
-- ssnf (WrapperConcat (MkConcat a b)) = Concat (ssnf a) (ssnf b)
ssnf (WrapperGroup (MkGroup a)) = Group (ssnf a)
ssnf (WrapperEps MkEps) = Eps
ssnf (WrapperNone MkNone) = None

ss (WrapperAlt (MkAlt a b)) = Alt (ss a) (ss b)
-- ss (WrapperConcat (MkConcat a b)) = 
-- if (isNull a) && (isNull b) 
--    then Alt (ss a) (ss b) 
--    else Concat (ssnf a) (ssnf b)
ss (WrapperChr (MkChr c)) = Chr c
ss (WrapperStar (MkStar a)) = ss a
ss (WrapperGroup (MkGroup a)) = Group (ss a)
ss (WrapperEps MkEps) = None
ss (WrapperNone MkNone) = None

-- merge : List Char -> List Char -> List Char
-- merge cs ds = let srt = sort (cs ++ ds) in makeUnique srt
--   where
--     makeUnique : List Char -> List Char
--     makeUnique cs = ?makeUnique_rhs


-- first : RegexTypes -> List Char
-- first (WrapperAlt (MkAlt a b)) = ?merge (first a) (first b)
-- first (WrapperChr (MkChr a)) = [a]
-- first (WrapperStar (MkStar a)) = first a
-- first (WrapperGroup (MkGroup a)) = first a
-- first (WrapperEps MkEps) = []
-- first (WrapperNone MkNone) = []
-- first (WrapperConcat (MkConcat (WrapperNone MkNone) b)) = first b
-- first (WrapperConcat (MkConcat a b)) = let x = first a in 
                                           -- if (isNull a)
                                           --    then merge x (first b)
                                           --    else x


-- collectConcatChain


-- concatAcceptence : Concat a b -> List Char -> List a


testAcceptance : List Char -> Type -> Maybe Bool
-- testAcceptance [] x = Just (isNull x)
-- testAcceptance (y :: xs) (WrapperChr (MkChr a)) = ?testAcceptance_rhs_10
-- testAcceptance (y :: xs) (WrapperStar (MkStar a)) = (testAcceptance [y] a) && (testAcceptance xs (WrapperStar (MkStar a)))
-- -- testAcceptance (y :: xs) (WrapperConcat (MkConcat a b)) = ?dfsfasd
--   -- if testAcceptance a 
--   --    then ?fdsfd
--   --    else ?fdsfsd
-- testAcceptance (y :: xs) (WrapperGroup (MkGroup a)) = ?testAcceptance_rhs_13k
-- testAcceptance (y :: xs) (WrapperEps MkEps) = False
-- testAcceptance (y :: xs) (WrapperNone MkNone) = False
--
-- a : Bool
-- a = testAcceptance (unpack "123") (WrapperStar (MkStar 
--     (WrapperAlt (MkAlt 
--     (WrapperAlt (MkAlt 
--     (WrapperChr (MkChr '1')) 
--     (WrapperChr (MkChr '2')))) 
--     (WrapperChr (MkChr '3'))))
--                    ))


-- makeRegexFromType : Type -> Maybe RegexTypes
-- makeRegexFromType (Alt a b) = Just (WrapperAlt (MkAlt (makeRegexFromType a) ?b1))
-- makeRegexFromType (Chr a) = ?fdsf2
-- makeRegexFromType (Star a) = ?fdsfd3
-- makeRegexFromType (Group a) = ?fdsfds5
-- makeRegexFromType Eps = ?fdsfds4
-- makeRegexFromType None  = ?fdsfsdfd6
-- makeRegexFromType _ = Nothing
-- ssnfEquiv : (str : List Char) -> (reg: RegexTypes) -> ((testAcceptance str reg) = (testAcceptance str (ssnf reg)))



-- alwaysZero : Nat -> Nat
-- alwaysZero 0 = 0
-- alwaysZero (S k) = 0
--
-- alwaysZero2 : Nat -> Nat
-- alwaysZero2 0 = 0
-- alwaysZero2 (S k) = 0
--
--
-- FuncEquivalence: (n : Nat) -> (alwaysZero n = alwaysZero2 n)
-- FuncEquivalence 0 = Refl
-- FuncEquivalence (S k) = Refl


