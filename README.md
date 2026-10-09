# Computer Management Web Application

> This project was made to satisfy the final term of Software Engineering.

## Table Of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)

## Overview

This repository is dedicated towards using Open Source Softwares to create the program,as well as document any hurdles along the way.

**Key Features**
- Can be self-hosted by the user
- Open Source
- Frontend/Backend is containerized for ease of deployment

## Architecture

The system has three **tiers**(what runs where, each has its own container). The backend is further organized into **layers**

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
        D[("Database")]
    end
    F -->|HTTP / JSON| C
    R -->|SQL| D
```

## Quick Start
```bash
git clone https://github.com/Narwhal1412/Software-Engineering-Finale.git
cd Software-Engineering-Finale
cp .env.example .env # then edit with your own values
docker compose up --build
```
