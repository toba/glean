import Foundation

class Session {
    let configuration: URLSessionConfiguration
    var activeTasks: [Int: DataRequest] = [:]

    init(configuration: URLSessionConfiguration = .default) {
        self.configuration = configuration
    }

    func request(_ url: String) -> DataRequest {
        let task = DataRequest(url: url)
        activeTasks[task.id] = task
        return task
    }

    func cancelAll() {
        for (_, task) in activeTasks {
            task.cancel()
        }
        activeTasks.removeAll()
    }
}
