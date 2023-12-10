package stack

import (
	"container/list"
)

type Stack struct {
	data *list.List
}

func NewStack() *Stack {
	return &Stack{data: list.New()}
}

func (s *Stack) Push(x interface{}) {
	s.data.PushBack(x)
}

func (s *Stack) Pop() interface{} {
	if s.data.Len() == 0 {
		return nil
	}
	tail := s.data.Back()
	val := tail.Value
	s.data.Remove(tail)
	return val
}
