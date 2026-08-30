# DC OS — Web

Versão web (React + Vite + Tailwind CSS v4) do dashboard do **DC Assistant**,
espelhando o design system "Midnight Cyan" da UI Slint em `firmware/ui/` e
falando com o backend `dc-os-core` em `backend/`.

## Stack

- React 19 + Vite
- Tailwind CSS v4 (`@tailwindcss/vite`)
- React Router
- lucide-react (ícones)

## Desenvolvimento

```bash
cd web
npm install
cp .env.example .env   # ajusta VITE_API_BASE_URL se necessário
npm run dev
```

A app assume o backend `dc-os-core` a correr em `http://localhost:8081`
(ver `backend/README.md` para o subir com `docker compose up -d --build`).
Sem o backend ligado, as páginas caem num estado "offline" com layout de
exemplo em vez de rebentar.

## Build de produção

```bash
npm run build
npm run preview
```

## Estrutura

```text
web/
├── src/
│   ├── components/     # StatusBar, Sidebar, Panel
│   ├── pages/           # Início, Assistente, Spotify, Clima, Recursos, Notas, Alarme, Definições
│   ├── lib/              # cliente API + hook de polling
│   ├── index.css         # tokens de design (portados de firmware/ui/theme.slint)
│   ├── App.jsx
│   └── main.jsx
└── vite.config.js
```

## Design

Paleta, raios e espaçamento replicam `firmware/ui/theme.slint` ("Midnight
Cyan"): fundos `#0A0E14 → #161C26`, accent ciano-azul `#38BDF8` e as mesmas
variações (`accent-cyan`, `accent-blue`, `accent-pink`, `accent-violet`),
para que a experiência web e o ecrã físico do ES3C28P pareçam a mesma
aplicação.
