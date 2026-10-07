package repositories

import (
	"gorm.io/gorm"
	"hello-service/models"
)

// MemberRepository exposes the data operations needed by the service layer.
type MemberRepository interface {
	FindAll() ([]models.Member, error)
}

type memberRepository struct {
	db *gorm.DB
}

func NewMemberRepository(db *gorm.DB) MemberRepository {
	return &memberRepository{db: db}
}

func (r *memberRepository) FindAll() ([]models.Member, error) {
	var members []models.Member
	if err := r.db.Order("id ASC").Find(&members).Error; err != nil {
		return nil, err
	}
	return members, nil
}
