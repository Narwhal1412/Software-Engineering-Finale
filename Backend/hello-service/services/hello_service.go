package services

import (
	"hello-service/models"
	"hello-service/repositories"
)

type HelloResponse struct {
	Message string          `json:"message"`
	Members []models.Member `json:"members"`
}

type HelloService struct {
	members repositories.MemberRepository
}

func NewHelloService(members repositories.MemberRepository) *HelloService {
	return &HelloService{members: members}
}

func (s *HelloService) GetHello() (HelloResponse, error) {
	members, err := s.members.FindAll()
	if err != nil {
		return HelloResponse{}, err
	}

	return HelloResponse{
		Message: "Hello from Go!",
		Members: members,
	}, nil
}
