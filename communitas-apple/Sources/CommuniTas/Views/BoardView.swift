import SwiftUI
import X0xClient

/// Kanban board with three columns: To Do, In Progress, Done.
struct BoardView: View {
    let groupId: String
    @EnvironmentObject var appState: AppState

    @State private var listId: String?
    @State private var tasks: [TaskItem] = []
    @State private var isLoading = true
    @State private var errorMessage: String?
    @State private var newTaskTitle = ""
    @State private var addingToColumn: String?
    @State private var pollTimer: Timer?

    private let columns = ["todo", "in_progress", "done"]

    var body: some View {
        VStack(spacing: 0) {
            boardHeader
            Divider()

            if isLoading {
                ProgressView("Loading board...")
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let errorMessage {
                errorView(errorMessage)
            } else {
                kanbanColumns
            }
        }
        .task {
            await loadOrCreateBoard()
            startPolling()
        }
        .onDisappear {
            pollTimer?.invalidate()
            pollTimer = nil
        }
    }

    private var boardHeader: some View {
        HStack {
            Image(systemName: "rectangle.3.group")
                .foregroundStyle(.secondary)
            Text("Board")
                .font(.headline)
            Spacer()
            Button {
                Task { await refreshTasks() }
            } label: {
                Label("Refresh", systemImage: "arrow.clockwise")
                    .font(.caption)
            }
            .buttonStyle(.bordered)
            .controlSize(.small)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.bar)
    }

    private var kanbanColumns: some View {
        HStack(alignment: .top, spacing: 12) {
            kanbanColumn(title: "To Do", status: "todo", icon: "circle", color: .secondary)
            kanbanColumn(title: "In Progress", status: "in_progress", icon: "clock", color: .orange)
            kanbanColumn(title: "Done", status: "done", icon: "checkmark.circle.fill", color: .green)
        }
        .padding(16)
    }

    private func kanbanColumn(title: String, status: String, icon: String, color: Color) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: icon)
                    .foregroundStyle(color)
                    .font(.caption)
                Text(title)
                    .font(.subheadline)
                    .fontWeight(.semibold)

                let count = tasks.filter { $0.status == status }.count
                Text("\(count)")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(Color.secondary.opacity(0.1), in: Capsule())

                Spacer()

                if status == "todo" {
                    Button {
                        addingToColumn = status
                    } label: {
                        Image(systemName: "plus")
                            .font(.caption)
                    }
                    .buttonStyle(.plain)
                    .foregroundStyle(.secondary)
                }
            }

            ScrollView {
                LazyVStack(spacing: 6) {
                    ForEach(tasks.filter { $0.status == status }) { task in
                        taskCard(task: task, status: status)
                    }

                    if addingToColumn == status {
                        addTaskField
                    }
                }
            }
        }
        .padding(12)
        .background(Color.secondary.opacity(0.04), in: RoundedRectangle(cornerRadius: 10))
        .frame(minWidth: 200)
    }

    private func taskCard(task: TaskItem, status: String) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(task.description)
                .font(.subheadline)
                .lineLimit(3)

            HStack(spacing: 4) {
                if status == "todo" {
                    Button("Claim") {
                        Task { await claimTaskAction(task) }
                    }
                    .font(.caption2)
                    .buttonStyle(.bordered)
                    .controlSize(.mini)
                } else if status == "in_progress" {
                    Button("Complete") {
                        Task { await completeTaskAction(task) }
                    }
                    .font(.caption2)
                    .buttonStyle(.bordered)
                    .controlSize(.mini)
                    .tint(.green)
                }

                Spacer()

                Text(String(task.taskId.prefix(6)))
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
                    .fontDesign(.monospaced)
            }
        }
        .padding(10)
        .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 8))
    }

    private var addTaskField: some View {
        VStack(spacing: 6) {
            TextField("Task title...", text: $newTaskTitle)
                .textFieldStyle(.plain)
                .padding(8)
                .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 6))
                .onSubmit { Task { await addTaskAction() } }

            HStack {
                Button("Add") {
                    Task { await addTaskAction() }
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
                .disabled(newTaskTitle.trimmingCharacters(in: .whitespaces).isEmpty)

                Button("Cancel") {
                    newTaskTitle = ""
                    addingToColumn = nil
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
            }
        }
        .padding(8)
        .background(Color.secondary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8))
    }

    private func errorView(_ message: String) -> some View {
        VStack(spacing: 12) {
            Image(systemName: "exclamationmark.triangle")
                .font(.system(size: 36))
                .foregroundStyle(.orange)
            Text(message)
                .foregroundStyle(.secondary)
            Button("Retry") {
                Task { await loadOrCreateBoard() }
            }
            .buttonStyle(.borderedProminent)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: - Actions

    private var boardStoreName: String {
        let prefix = appState.groupPrefix(for: groupId)
        return "x0x-board-\(prefix)"
    }

    private func ensureBoardStore() async {
        do {
            let stores = try await appState.client.listStores()
            if !stores.contains(where: { $0.storeId == boardStoreName || $0.name == boardStoreName }) {
                let prefix = appState.groupPrefix(for: groupId)
                _ = try await appState.client.createStore(name: boardStoreName, topic: "x0x.group.\(prefix).board")
            }
        } catch { /* store may already exist */ }
    }

    private func loadOrCreateBoard() async {
        isLoading = true
        errorMessage = nil
        let prefix = appState.groupPrefix(for: groupId)
        let kvKey = "board.\(prefix).listId"

        await ensureBoardStore()

        do {
            let storedId = try await appState.client.storeGet(storeId: boardStoreName, key: kvKey)
            listId = storedId
            await refreshTasks()
        } catch {
            // No board yet, create one
            do {
                let topic = "x0x.group.\(prefix).board/tasks"
                let newId = try await appState.client.createTaskList(name: "Board", topic: topic)
                try await appState.client.storePut(storeId: boardStoreName, key: kvKey, value: newId)
                listId = newId
                tasks = []
            } catch {
                errorMessage = "Failed to create board: \(error.localizedDescription)"
            }
        }
        isLoading = false
    }

    private func refreshTasks() async {
        guard let listId else { return }
        do {
            tasks = try await appState.client.listTasks(listId: listId)
        } catch {
            // Silently ignore refresh failures during polling
        }
    }

    private func addTaskAction() async {
        let title = newTaskTitle.trimmingCharacters(in: .whitespaces)
        guard !title.isEmpty, let listId else { return }
        newTaskTitle = ""
        addingToColumn = nil

        do {
            _ = try await appState.client.addTask(listId: listId, title: title)
            await refreshTasks()
        } catch {
            errorMessage = "Failed to add task: \(error.localizedDescription)"
        }
    }

    private func claimTaskAction(_ task: TaskItem) async {
        guard let listId else { return }
        do {
            try await appState.client.claimTask(listId: listId, taskId: task.taskId)
            await refreshTasks()
        } catch {
            errorMessage = "Failed to claim task: \(error.localizedDescription)"
        }
    }

    private func completeTaskAction(_ task: TaskItem) async {
        guard let listId else { return }
        do {
            try await appState.client.completeTask(listId: listId, taskId: task.taskId)
            await refreshTasks()
        } catch {
            errorMessage = "Failed to complete task: \(error.localizedDescription)"
        }
    }

    private func startPolling() {
        pollTimer?.invalidate()
        pollTimer = Timer.scheduledTimer(withTimeInterval: 5.0, repeats: true) { _ in
            Task { @MainActor in
                await refreshTasks()
            }
        }
    }
}
