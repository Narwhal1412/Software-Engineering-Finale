package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"hello-service/services"
)

type HelloHandler struct {
	service *services.HelloService
}

func NewHelloHandler(service *services.HelloService) *HelloHandler {
	return &HelloHandler{service: service}
}

func (h *HelloHandler) GetHello(c *gin.Context) {
	response, err := h.service.GetHello()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"message": "failed to load members",
		})
		return
	}

	c.JSON(http.StatusOK, response)
}
