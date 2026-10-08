package main

import (
	"fmt"
	"log"
	"time"

	"hello-service/config"
	"hello-service/handlers"
	"hello-service/models"
	"hello-service/repositories"
	"hello-service/routes"
	"hello-service/services"

	"github.com/gin-gonic/gin"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

func connectDatabase(cfg config.Config) (*gorm.DB, error) {
	dsn := fmt.Sprintf(
		"host=%s port=%s user=%s password=%s dbname=%s sslmode=%s TimeZone=Asia/Ho_Chi_Minh",
		cfg.DBHost, cfg.DBPort, cfg.DBUser, cfg.DBPassword, cfg.DBName, cfg.DBSSLMode,
	)

	var db *gorm.DB
	var err error

	// PostgreSQL starts slightly later than the API container, so retry a few times.
	for attempt := 1; attempt <= 15; attempt++ {
		db, err = gorm.Open(postgres.Open(dsn), &gorm.Config{})
		if err == nil {
			sqlDB, dbErr := db.DB()
			if dbErr == nil {
				if pingErr := sqlDB.Ping(); pingErr == nil {
					return db, nil
				} else {
					err = pingErr
				}
			} else {
				err = dbErr
			}
		}

		log.Printf("database connection attempt %d/15 failed: %v", attempt, err)
		time.Sleep(2 * time.Second)
	}

	return nil, fmt.Errorf("unable to connect to database: %w", err)
}

func main() {
	cfg := config.Load()

	db, err := connectDatabase(cfg)
	if err != nil {
		log.Fatal(err)
	}

	if err := db.AutoMigrate(&models.Member{}); err != nil {
		log.Fatal(err)
	}

	memberRepository := repositories.NewMemberRepository(db)
	helloService := services.NewHelloService(memberRepository)
	helloHandler := handlers.NewHelloHandler(helloService)

	router := gin.Default()
	routes.Register(router, helloHandler)

	log.Printf("server listening on :%s", cfg.AppPort)
	if err := router.Run(":" + cfg.AppPort); err != nil {
		log.Fatal(err)
	}
}
