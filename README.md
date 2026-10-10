# Computer Management Web Application

> This project was made to satisfy the final term of Software Engineering.

## Table Of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Repository Structure](#repository-structure)
- [Prerequisites](#Prerequisites)
- [Quick Start](#quick-start)
- [Environment Variable](#environment-variables)
- [Documentation](#documentation)

## Overview

This repository is dedicated towards using Open Source Softwares to create the program,as well as document any hurdles along the way.

**Key Features**
- Can be self-hosted by the user
- Open Source
- Frontend/Backend is containerized for ease of deployment

## Architecture

The system has three **tiers**(what runs where, each has its own container). The backend is further organized into **layers**.

```mermaid
flowchart LR
    subgraph T1["Presentation tier"]
        F["Frontend<br/>Presentation layer"]
    end
    subgraph T2["Application tier"]
        subgraph B["Backend"]
            C["Controllers / routes"] --> S["Services<br/>Business Logic layer"] --> R["Repositories<br/>Data Access layer"]
        end
    end
    subgraph T3["Data tier"]
        D[("PostgreSQL")]
    end
    F -->|HTTP / JSON| C
    R -->|SQL| D
 
    classDef pres fill:#dbeafe,stroke:#2563eb,color:#1e3a8a
    classDef logic fill:#dcfce7,stroke:#16a34a,color:#14532d
    classDef data fill:#fef3c7,stroke:#d97706,color:#78350f
    class F pres
    class C,S,R logic
    class D data
 
    style T1 fill:#eff6ff,stroke:#2563eb,color:#1e3a8a
    style T2 fill:#f0fdf4,stroke:#16a34a,color:#14532d
    style B fill:#dcfce7,stroke:#16a34a,color:#14532d
    style T3 fill:#fffbeb,stroke:#d97706,color:#78350f
```

| Tier | Layer(s) | Technology | Details |
|------|----------|------------|---------|
| Presentation | Presentation | ... | [Frontend/README.md](Frontend/README.md) |
| Application | Business Logic, Data Access | ... | [Backend/README.md](Backend/README.md) |
| Data | none (data store) | PostgreSQL | defined in `compose.yaml` |

## Repository Structure

```
.
├── Backend/          	  # API service
├── Frontend/         	  # Web UI
├── .woodpecker 	  # CI/CD Pipeline instructions yaml files
├── docs/             	  # Plans, notes, design decisions
├── .env.example      	  # Template for required environment variables
├── compose.yaml      	  # Defines how the services run together
├── compose.override.yaml # Override ports mapping for local deployment
├── compose.prod.yaml 	  # Override for adding labels for CI/CD
├── .gitignore 		  # Files ignored(.env) when pushed to git 
└── README.md
```

## Prerequisites

- [Git](https://git-scm.com)
- [Docker](https://docs.docker.com/get-started/get-docker/)

## Quick Start
```bash
git clone https://github.com/Narwhal1412/Software-Engineering-Finale.git
cd Software-Engineering-Finale
cp .env.example .env # then edit with your own values
docker compose up --build
```
| Service | URL |
|---------|-----|
| Frontend | http://localhost:<port> |
| Backend API | http://localhost:<port> |

Stop everything with `docker compose down`.

## Environment Variables

Copy `.env.example` to `.env` and fill in the values.

```
cp .env.example .env
```
| Variable | Description | Example |
|----------|-------------|---------|
| `DB_USER` | Database username | `app` |
| `DB_PASSWORD` | Database password | `change-me` |
| `DB_NAME` | Database name | `appdb` |
| `APP_PORT` | Port the backend listens on | `8080` |
 
## Documentation

