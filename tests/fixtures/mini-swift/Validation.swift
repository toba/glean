import Foundation

let acceptableStatusCodes = 200..<300

struct Validation {
    static func validate(request: DataRequest) -> DataRequest {
        guard let response = request.response else {
            return request
        }
        if acceptableStatusCodes.contains(response.statusCode) {
            return request
        }
        return request
    }
}
