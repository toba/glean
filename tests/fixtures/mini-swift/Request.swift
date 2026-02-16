import Foundation

protocol RequestDelegate {
    func didComplete(request: DataRequest)
}

class DataRequest {
    static var nextId = 0

    let id: Int
    let url: String
    var response: HTTPURLResponse?
    var delegate: RequestDelegate?

    init(url: String) {
        DataRequest.nextId += 1
        self.id = DataRequest.nextId
        self.url = url
    }

    func cancel() {
        // Cancel the underlying task
    }

    func validate() -> DataRequest {
        return Validation.validate(request: self)
    }
}
