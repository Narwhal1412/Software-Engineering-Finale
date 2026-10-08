package routes

import (
	"hello-service/handlers"
	"net/http"

	"github.com/gin-gonic/gin"
)

func Register(router *gin.Engine, helloHandler *handlers.HelloHandler) {
	router.GET("/health", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	api := router.Group("/api/v1")
	api.GET("/hello", helloHandler.GetHello)
}
