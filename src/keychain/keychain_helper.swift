import LocalAuthentication
import Foundation

enum Status: Int32 {
    case success = 0
    case failed = 1
}

@_cdecl("add_token")
public func addToken(_ keyPtr: UnsafePointer<CChar>, _ tokenPtr: UnsafePointer<CChar>) -> Int32 {
    let keyString = String(cString: keyPtr)
    let tokenString = String(cString: tokenPtr)
    
    guard let tokenData = tokenString.data(using: .utf8) else {
        return Status.failed.rawValue
    }
        
    let deleteQuery: [String: Any] = [
        kSecClass as String: kSecClassGenericPassword,
        kSecAttrAccount as String: keyString,
        kSecAttrService as String: "com.thenicekat.srs"
    ]
    
    SecItemDelete(deleteQuery as CFDictionary)
    
    let addQuery: [String: Any] = [
        kSecClass as String: kSecClassGenericPassword,
        kSecAttrAccount as String: keyString,
        kSecAttrService as String: "com.thenicekat.srs",
        kSecValueData as String: tokenData,
        kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlocked
    ]
    
    let status = SecItemAdd(addQuery as CFDictionary, nil)
    if status == errSecSuccess {
        return Status.success.rawValue
    } else {
        return Status.failed.rawValue
    }
}

@_cdecl("get_token")
public func getToken(_ keyPtr: UnsafePointer<CChar>) -> UnsafePointer<CChar>? {
    let keyString = String(cString: keyPtr)
    
    let query: [String: Any] = [
        kSecClass as String: kSecClassGenericPassword,
        kSecAttrAccount as String: keyString,
        kSecAttrService as String: "com.thenicekat.srs",
        kSecReturnData as String: true,
        kSecMatchLimit as String: kSecMatchLimitOne
    ]
    
    var item: CFTypeRef?
    let status = SecItemCopyMatching(query as CFDictionary, &item)
    
    if status == errSecSuccess, let data = item as? Data, let tokenString = String(data: data, encoding: .utf8) {
        if let result = strdup(tokenString) {
            return UnsafePointer(result)
        }
    }
    
    return nil
}

@_cdecl("list_tokens")
public func listTokens() -> UnsafePointer<CChar>? {
    let query: [String: Any] = [
        kSecClass as String: kSecClassGenericPassword,
        kSecAttrService as String: "com.thenicekat.srs",
        kSecReturnAttributes as String: true,
        kSecMatchLimit as String: kSecMatchLimitAll
    ]
    
    var items: CFTypeRef?
    let status = SecItemCopyMatching(query as CFDictionary, &items)
    
    if status == errSecSuccess, let itemsArray = items as? [[String: Any]] {
        let keys = itemsArray.compactMap { $0[kSecAttrAccount as String] as? String }
        if let jsonData = try? JSONSerialization.data(withJSONObject: keys, options: []),
           let jsonString = String(data: jsonData, encoding: .utf8),
           let result = strdup(jsonString) {
            return UnsafePointer(result)
        }
    }
    
    if let result = strdup("[]") {
        return UnsafePointer(result)
    }
    
    return nil
}

@_cdecl("delete_token")
public func deleteToken(_ keyPtr: UnsafePointer<CChar>) -> Int32 {
    let keyString = String(cString: keyPtr)
    
    let query: [String: Any] = [
        kSecClass as String: kSecClassGenericPassword,
        kSecAttrAccount as String: keyString,
        kSecAttrService as String: "com.thenicekat.srs"
    ]
    
    let status = SecItemDelete(query as CFDictionary)
    
    if status == errSecSuccess || status == errSecItemNotFound {
        return Status.success.rawValue
    } else {
        return Status.failed.rawValue
    }
}
