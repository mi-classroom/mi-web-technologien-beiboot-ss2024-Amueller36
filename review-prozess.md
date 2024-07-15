# GitHub Review Prozess

Dieser Review-Prozess beschreibt den Workflow für das Erstellen und Bearbeiten von Issues, das Arbeiten in separaten Branches und das Erstellen und Überprüfen von Pull Requests (PRs) in diesem Repository.

## Workflow

1. **Issue erstellen**
   - Bei neuen Features oder Bugs ein neues Issue im GitHub-Repository anlegen.
   - Das Issue sollte eine klare und prägnante Beschreibung des Features oder Bugs enthalten.
   - Labels verwenden, um den Typ des Issues zu kennzeichnen (z.B. `bug`, `enhancement`, `question`).

2. **Branch erstellen**
   - Einen neuen Branch vom `main`-Branch erstellen, der den Namen des Issues enthält. 
     - Beispiel: `feature/issue-#123-feature-name` oder `bugfix/issue-#456-bug-description`
   - In diesem Branch arbeiten und alle Änderungen commiten.

3. **Pull Request erstellen**
   - Sobald die Arbeit abgeschlossen ist, einen Pull Request (PR) vom erstellten Branch zum `main`-Branch stellen.
   - Der PR sollte eine Beschreibung der durchgeführten Änderungen enthalten und das zugehörige Issue referenzieren (z.B. `Fixes #123`).

4. **Code Review**
   - Ein anderer Student (Review Buddy) wird als Reviewer zugewiesen.
   - Der Reviewer prüft den PR anhand der folgenden Kriterien:

     ### Review-Kriterien

     - **Code Qualität**
       - Ist der Code klar und verständlich?
       - Werden Best Practices und Coding-Standards eingehalten?
       - Gibt es unnötigen oder redundanten Code?

     - **Funktionalität**
       - Erfüllt der Code die Anforderungen des Issues?
       - Wurden alle im Issue beschriebenen Änderungen umgesetzt?
       - Funktioniert der Code wie erwartet? (Manuelle Tests durchführen)

     - **Dokumentation**
       - Ist der Code ausreichend kommentiert?
       - Wurden alle neuen oder geänderten Funktionen in der Dokumentation festgehalten?
       - Gibt es README- oder Changelog-Updates, wenn erforderlich?


5. **PR Freigabe und Merge**
   - Nach erfolgreicher Überprüfung und Genehmigung durch den Reviewer wird der PR in den `main`-Branch gemergt.
   - Das zugehörige Issue wird geschlossen und der Branch gelöscht

## Rollen und Verantwortlichkeiten

- **Autor**: Erstellt und arbeitet am Branch, erstellt den PR und stellt sicher, dass alle Anforderungen erfüllt sind.
- **(Review Buddy)**: Prüft den PR gründlich anhand der oben genannten Kriterien und gibt Feedback oder genehmigt den PR.
