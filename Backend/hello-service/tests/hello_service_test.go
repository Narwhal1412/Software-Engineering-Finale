package tests

import (
	"errors"
	"testing"

	"hello-service/models"
	"hello-service/repositories"
	"hello-service/services"
)

type fakeMemberRepository struct {
	members []models.Member
	err     error
}

func (f *fakeMemberRepository) FindAll() ([]models.Member, error) {
	if f.err != nil {
		return nil, f.err
	}
	return f.members, nil
}

var _ repositories.MemberRepository = (*fakeMemberRepository)(nil)

func TestGetHello(t *testing.T) {
	repo := &fakeMemberRepository{
		members: []models.Member{{ID: 1, Name: "Member 1"}, {ID: 2, Name: "Member 2"}},
	}

	service := services.NewHelloService(repo)
	response, err := service.GetHello()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if response.Message != "Hello from Go!" {
		t.Fatalf("unexpected message: %s", response.Message)
	}

	if len(response.Members) != 2 {
		t.Fatalf("expected 2 members, got %d", len(response.Members))
	}
}

func TestGetHelloReturnsRepositoryError(t *testing.T) {
	repo := &fakeMemberRepository{err: errors.New("database unavailable")}
	service := services.NewHelloService(repo)

	if _, err := service.GetHello(); err == nil {
		t.Fatal("expected error, got nil")
	}
}
