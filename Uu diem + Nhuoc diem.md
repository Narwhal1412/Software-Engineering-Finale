# Framework & Technology Comparison for a Go Product Skeleton

## 1. Mục tiêu

Tài liệu này so sánh các lựa chọn phổ biến để xây dựng một sản phẩm web với **Go ở backend**, frontend web và database. Mục tiêu của skeleton là **đơn giản, dễ hiểu, dễ mở rộng và đủ tốt cho sản phẩm thử nghiệm**.

Chức năng thử nghiệm hiện tại chỉ cần:

```text
Frontend
   ↓ HTTP/JSON
Go Backend
   ↓
Database
   ↓
Danh sách tên thành viên
```

API mẫu:

```text
GET /health
GET /api/v1/members
```

---

# 2. Backend – các framework/router Go

## 2.1 Gin

**Đặc điểm:** framework web phổ biến cho Go, cung cấp routing, middleware, JSON binding và validation.

### Ưu điểm

- API dễ học và cú pháp trực quan.
- Routing và middleware rõ ràng.
- Hỗ trợ binding/validation cho request.
- Phù hợp REST API và microservice.
- Hệ sinh thái package hỗ trợ khá rộng.
- Dễ tổ chức theo Handler → Service → Repository.

### Nhược điểm

- Không phải full-stack framework nên database, authentication, migration, logging... vẫn phải tự chọn.
- Có thể bị tổ chức thành nhiều layer/package nếu over-engineer.
- Nếu project rất nhỏ, một số abstraction có thể là dư thừa.

### Đánh giá cho skeleton

**Rất phù hợp – lựa chọn đề xuất.**

Gin phù hợp với mục tiêu “dễ hiểu nhưng vẫn đủ thực tế” của skeleton.

Nguồn chính thức: https://gin-gonic.com/en/docs/

---

## 2.2 Fiber

**Đặc điểm:** framework Go có API lấy cảm hứng từ Express và xây dựng trên Fasthttp, tập trung vào trải nghiệm phát triển nhanh và hiệu năng.

### Ưu điểm

- Cú pháp khá quen thuộc với người đã dùng Express.
- API gọn.
- Hiệu năng được ưu tiên mạnh.
- Có nhiều middleware và integration.
- Phù hợp API và service cần throughput cao.

### Nhược điểm

- Khác với hệ sinh thái `net/http` chuẩn của Go nhiều hơn so với các lựa chọn bám sát standard library.
- Người mới học Go có thể hình thành thói quen theo phong cách Express thay vì hiểu sâu `net/http`.
- Một số đặc điểm của context/request cần chú ý vì Fiber tối ưu và tái sử dụng dữ liệu request.

### Đánh giá cho skeleton

**Tốt**, đặc biệt khi team đã quen Express.

Nguồn chính thức: https://docs.gofiber.io/

---

## 2.3 Echo

**Đặc điểm:** framework Go theo hướng minimalist/high-performance, có router và nhiều middleware tích hợp.

### Ưu điểm

- API rõ ràng.
- Router hiệu năng cao.
- Có nhiều middleware sẵn.
- Hỗ trợ xây dựng REST API nhanh.
- Hệ sinh thái có JWT, Swagger/OpenAPI và các integration phổ biến.

### Nhược điểm

- Vẫn cần tự thiết kế architecture, data access, migration và business layer.
- Nếu mục tiêu là học Go theo standard library càng gần càng tốt thì `net/http` hoặc chi có thể phù hợp hơn.
- Project nhỏ có thể không cần nhiều tính năng tích hợp sẵn.

### Đánh giá cho skeleton

**Tốt**, là một lựa chọn thay thế trực tiếp cho Gin.

Nguồn chính thức: https://echo.labstack.com/

---

## 2.4 Chi

**Đặc điểm:** router nhẹ, idiomatic và composable; tương thích với `net/http`.

### Ưu điểm

- Rất nhẹ.
- Bám sát Go standard library.
- Middleware và router có tính composable cao.
- Phù hợp REST API lớn cần maintainability.
- Giảm lượng abstraction mà framework áp đặt.

### Nhược điểm

- Ít “batteries included” hơn các framework như Gin/Fiber/Echo.
- Phải tự chọn thêm nhiều thành phần cho validation, binding, API documentation...
- Người mới có thể phải hiểu `net/http` nhiều hơn trước khi cảm thấy thoải mái.

### Đánh giá cho skeleton

**Rất tốt nếu ưu tiên Go thuần và kiến trúc nhẹ**, nhưng không phải lựa chọn dễ nhất cho người mới.

Nguồn chính thức: https://github.com/go-chi/chi

---

## 2.5 So sánh backend

| Tiêu chí | Gin | Fiber | Echo | Chi |
|---|---:|---:|---:|---:|
| Dễ học | ★★★★★ | ★★★★☆ | ★★★★☆ | ★★★☆☆ |
| API gọn | ★★★★★ | ★★★★★ | ★★★★★ | ★★★★☆ |
| Middleware | ★★★★★ | ★★★★★ | ★★★★★ | ★★★★☆ |
| Bám sát Go stdlib | ★★★★☆ | ★★☆☆☆ | ★★★★☆ | ★★★★★ |
| REST API | ★★★★★ | ★★★★★ | ★★★★★ | ★★★★★ |
| Microservice | ★★★★★ | ★★★★★ | ★★★★★ | ★★★★★ |
| Phù hợp người mới | ★★★★★ | ★★★★☆ | ★★★★☆ | ★★★☆☆ |
| Skeleton đề xuất | **YES** | Có thể | Có thể | Có thể |

### Kết luận backend

**Chọn Gin** cho skeleton hiện tại.

Lý do chính không phải chỉ vì hiệu năng, mà vì nó cân bằng tốt giữa:

```text
Dễ học
   +
Dễ tổ chức
   +
Đủ middleware
   +
REST API tốt
   +
Dễ mở rộng
```

---

# 3. Frontend – các lựa chọn

> Lưu ý: React là UI library, trong khi Vue và Angular được mô tả là framework. Trong thực tế, các lựa chọn này thường được đặt cạnh nhau khi chọn stack frontend. Vite là **build tool/dev server**, không phải UI framework.

## 3.1 React

### Ưu điểm

- Component-based.
- Hệ sinh thái lớn.
- Dễ chia thành `components`, `pages`, `services`.
- Phù hợp SPA và nhiều loại web application.
- Dễ kết nối REST API từ Go backend.

### Nhược điểm

- Bản thân React tập trung vào UI nên routing, state management, data fetching... có thể cần thêm công cụ tùy project.
- Có nhiều lựa chọn trong ecosystem nên người mới dễ bị phân tán.
- Cần quy ước architecture để project lớn không trở nên thiếu nhất quán.

### Đánh giá

**Lựa chọn đề xuất cho skeleton.**

Nguồn chính thức: https://react.dev/

---

## 3.2 Vue

### Ưu điểm

- Cú pháp khá gần HTML/CSS/JavaScript cơ bản.
- Component-based.
- Có thể áp dụng dần từ project nhỏ tới SPA/full application.
- Documentation dễ tiếp cận.

### Nhược điểm

- Nếu team đã quen React thì chuyển sang Vue không đem lại lợi ích rõ ràng cho skeleton đơn giản.
- Ecosystem và cách tổ chức project vẫn cần quy ước khi ứng dụng lớn.

### Đánh giá

**Rất tốt cho người mới**, là lựa chọn thay thế hợp lý cho React.

Nguồn chính thức: https://vuejs.org/guide/introduction.html

---

## 3.3 Angular

### Ưu điểm

- Framework đầy đủ và có tính opinionated cao.
- Có nhiều công cụ/API tích hợp.
- Dependency Injection hỗ trợ modularity và testability.
- Phù hợp codebase và team lớn cần conventions rõ ràng.

### Nhược điểm

- Nặng và nhiều khái niệm hơn cho một skeleton chỉ hiển thị danh sách thành viên.
- Learning curve cao hơn React/Vue cho project nhỏ.
- Dễ dẫn tới over-engineering nếu chỉ cần frontend test.

### Đánh giá

**Không ưu tiên cho skeleton hiện tại.**

Nguồn chính thức: https://angular.dev/docs

---

## 3.4 Svelte / SvelteKit

### Ưu điểm

- Component rất gọn.
- Svelte dùng compiler để chuyển component thành JavaScript tối ưu cho browser.
- Có SvelteKit làm application framework chính thức.
- Có thể tạo UI với ít boilerplate.

### Nhược điểm

- Hệ sinh thái và mức độ phổ biến trong team có thể không thuận lợi bằng React.
- Team mới phải học một syntax/tooling khác nếu chưa dùng Svelte.

### Đánh giá

**Tốt**, nhưng không cần thiết cho skeleton này.

Nguồn chính thức: https://svelte.dev/

---

## 3.5 Vite

Vite không thay thế React/Vue/Svelte. Nó là **build tool + development server** cho frontend.

### Ưu điểm

- Dev server khởi động nhanh.
- HMR nhanh.
- Cấu hình tương đối gọn.
- Hỗ trợ nhiều framework frontend.

### Nhược điểm

- Không phải UI framework.
- Vẫn phải chọn React/Vue/Svelte/... ở phía trên.

### Đánh giá

**Nên dùng Vite** cùng React cho skeleton frontend hiện tại.

Nguồn chính thức: https://vite.dev/guide/

---

# 4. Database – lựa chọn cho skeleton

Database không phải framework, nhưng cần lựa chọn cùng lúc với backend.

## 4.1 PostgreSQL

### Ưu điểm

- Relational database mạnh.
- ACID và transaction.
- Foreign key, constraint, index và nhiều tính năng SQL mạnh.
- Hỗ trợ JSON/JSONB bên cạnh mô hình relational.
- Phù hợp dữ liệu có quan hệ rõ như User, Payment, Order, Tuition.

### Nhược điểm

- Cần hiểu SQL, schema và relationship.
- Schema relational có thể cần migration khi thay đổi cấu trúc.
- Với dữ liệu hoàn toàn không đồng nhất, document database có thể đơn giản hơn.

### Đánh giá

**Lựa chọn đề xuất.**

Nguồn chính thức: https://www.postgresql.org/about/

---

## 4.2 MySQL

### Ưu điểm

- RDBMS phổ biến.
- Transaction-safe và ACID.
- Phù hợp web application và OLTP.
- Dễ tìm tài liệu và công cụ hỗ trợ.

### Nhược điểm

- Với một số bài toán dữ liệu phức tạp, PostgreSQL thường cho hệ tính năng SQL/extension phong phú hơn.
- Chuyển đổi giữa MySQL và PostgreSQL có thể phát sinh khác biệt về SQL/dialect và tính năng.

### Đánh giá

**Tốt**, nhưng không cần thiết nếu skeleton đã chốt PostgreSQL.

Nguồn chính thức: https://www.mysql.com/products/enterprise/database/

---

## 4.3 MongoDB

### Ưu điểm

- Document model linh hoạt.
- Schema có tính linh hoạt cao.
- Dữ liệu dạng document gần với JSON/object trong application code.
- Hỗ trợ transaction, replication và horizontal scaling.

### Nhược điểm

- Không phải lựa chọn tự nhiên nhất cho domain có rất nhiều quan hệ relational và constraint.
- Thiết kế data model khác RDBMS nên người đã quen SQL cần học cách tư duy mới.
- Với User–Payment–Order–Tuition có nhiều relationship, relational database thường dễ diễn đạt hơn.

### Đánh giá

**Tốt cho document-oriented data**, nhưng không phải lựa chọn ưu tiên cho skeleton này.

Nguồn chính thức: https://www.mongodb.com/docs/manual/

---

# 5. ORM / Database Access trong Go

## GORM

### Ưu điểm

- API ORM dễ tiếp cận.
- Giảm lượng SQL boilerplate cho CRUD thông thường.
- Hỗ trợ associations, transactions và migrations.
- Dễ kết hợp với layer Repository.

### Nhược điểm

- ORM thêm abstraction trên SQL.
- Query phức tạp đôi khi cần quay về SQL hoặc cách viết low-level hơn.
- Nếu lạm dụng ORM, developer có thể không hiểu rõ query thực sự chạy xuống database.

### Đánh giá

**Chọn GORM cho skeleton học tập/prototype.**

Khi project cần kiểm soát SQL cực kỳ chặt chẽ, có thể cân nhắc `database/sql` hoặc `sqlc`.

Nguồn: https://gorm.io/docs/

---

# 6. Stack được lựa chọn cho sản phẩm hiện tại

## Backend

```text
Go
└── Gin
    ├── Handler
    ├── Service
    ├── Repository
    └── GORM
```

## Frontend

```text
React
└── Vite
```

## Database

```text
PostgreSQL
```

## Container

```text
Docker
└── Docker Compose
```

## Testing

```text
Go testing
└── Testify (khi cần assertions/mocking)
```

---

# 7. Vì sao stack này phù hợp với skeleton?

Mục tiêu sản phẩm hiện tại rất nhỏ:

```text
Hello
+
Read members from database
```

Không cần ngay từ đầu:

```text
Kafka
Redis
GraphQL
Kubernetes
CQRS
Event Sourcing
API Gateway phức tạp
```

Stack tối thiểu nên là:

```text
React + Vite
      ↓
   REST API
      ↓
   Go + Gin
      ↓
   GORM
      ↓
 PostgreSQL
```

Sau này có thể mở rộng từng phần mà không phải thay toàn bộ skeleton.

---

# 8. Quyết định cuối cùng

| Thành phần | Lựa chọn | Lý do |
|---|---|---|
| Language | **Go** | Nhanh, gọn, phù hợp backend/service |
| Backend | **Gin** | Dễ học, routing/middleware tốt, REST API phù hợp |
| Frontend | **React** | Component-based, ecosystem lớn |
| Build tool | **Vite** | Dev/build nhanh và gọn |
| Database | **PostgreSQL** | Relational, transaction, constraint, phù hợp domain nghiệp vụ |
| ORM | **GORM** | CRUD nhanh, dễ tổ chức Repository |
| Container | **Docker Compose** | Dễ chạy backend + frontend + database |
| Testing | **Go testing** | Có sẵn trong Go; có thể thêm Testify |

---

# 9. Kết luận

Đối với sản phẩm thử nghiệm hiện tại, lựa chọn cân bằng nhất là:

**Go + Gin + GORM + PostgreSQL + React + Vite + Docker Compose**.

Không có framework nào tốt nhất cho mọi project. Lựa chọn nên dựa trên:

```text
Độ phức tạp của sản phẩm
+
Kinh nghiệm của team
+
Yêu cầu hiệu năng
+
Khả năng bảo trì
+
Hệ sinh thái
+
Thời gian phát triển
```

Với skeleton hiện tại, **Gin + React/Vite + PostgreSQL** giúp giữ project đủ nhỏ để học và test, nhưng vẫn có đường mở rộng rõ ràng khi thêm authentication, payment, notification, tuition và các service khác.

---

# 10. Official References

- Gin: https://gin-gonic.com/en/docs/
- Fiber: https://docs.gofiber.io/
- Echo: https://echo.labstack.com/
- Chi: https://github.com/go-chi/chi
- React: https://react.dev/
- Vue: https://vuejs.org/guide/introduction.html
- Angular: https://angular.dev/docs
- Svelte: https://svelte.dev/
- Vite: https://vite.dev/guide/
- PostgreSQL: https://www.postgresql.org/about/
- MySQL: https://www.mysql.com/products/enterprise/database/
- MongoDB: https://www.mongodb.com/docs/manual/
- GORM: https://gorm.io/docs/
