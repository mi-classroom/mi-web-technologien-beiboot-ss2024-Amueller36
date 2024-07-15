[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-24ddc0f5d75046c5622901739e7c5dd533143b0c8e959d652212380cedb1ea36.svg)](https://classroom.github.com/a/gQyBcnrC)

# Web Technologien // begleitendes Projekt Sommersemester 2024

Warum ist das cool? Bilder mit Langzeitbelichtung sind gar nicht so einfach zu erstellen, vor allem, wenn man möglichst
viel Kontrolle über das Endergebnis haben möchte. In unserem Ansatz, bildet ein Film den Ausgangspunkt. Diesen zerlegen
wir in Einzelbilder und montieren die Einzelbilder mit verschiedenen Blendmodes zu einem Bild mit
Langzeitbelichtungseffekt zusammen.

# Wie startet man das Projekt?
Das Projekt kann auf zwei Weisen gestartet werden, entweder mit Docker-Compose(empfohlen) oder Nativ.

## Wie führt man das Projekt mit Docker Compose aus?
Man muss Docker Compose installiert haben und kann dann in diesem "Root" Verzeichnis 

```sh
docker-compose up -d
```
ausführen und erreicht das Frontend dann unter `localhost:80`.

## Wie führt man das Projekt Nativ aus?

## Frontend
Das Frontend ist in Vue 3 entwickelt.
Um das Frontend Nativ zu starten muss man in dem `frontend` Ordner sein und folgenden Befehl ausführen:
```sh
npm install && npm run dev
```
Das Frontend ist dann unter `localhost:8081` erreichbar.

## Backend
Das Backend ist in Rust geschrieben und setzt die Rust Toolchain voraus und kann folgendermaßen gestartet werden.
Dazu muss man in dem `backend` Ordner sein und folgenden Befehl ausführen:

```sh
cargo run --release
```

# Reviewprozess
Der Reviewprozess für das Repository ist in der [review-prozess.md](review-prozess.md) zu finden.

# Zeitaufwand
Der Zeitaufwand für dieses Projekt ist in der [time.md](time.md) zu finden.

