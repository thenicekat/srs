import LocalAuthentication
import Foundation

enum Status: Int32 {
    case success = 0
    case failed = 1
}

@_cdecl("add_token")
public func addToken(_ keyPtr: UnsafePointer<CChar>, _ tokenPtr: UnsafePointer<CChar>) -> Int32 {
    print("SWIFT DEBUG: addToken called")
    let keyString = String(cString: keyPtr)
    let tokenString = String(cString: tokenPtr)
    print("SWIFT DEBUG: Adding token with key: '\(keyString)'")
    
    guard let tokenData = tokenString.data(using: .utf8) else {
        print("SWIFT DEBUG: Failed to convert token to data")
        return Status.failed.rawValue
    }
        
    let deleteQuery: [CFString: Any] = [
        kSecClass: kSecClassGenericPassword,
        kSecAttrAccount: keyString,
        kSecAttrService: "com.thenicekat.srs"
    ]
    
    let deleteStatus = SecItemDelete(deleteQuery as CFDictionary)
    print("SWIFT DEBUG: Delete existing item status: \(deleteStatus)")
    
    let addQuery: [CFString: Any] = [
        kSecClass: kSecClassGenericPassword,
        kSecAttrAccount: keyString,
        kSecAttrService: "com.thenicekat.srs",
        kSecValueData: tokenData,
        kSecAttrAccessible: kSecAttrAccessibleWhenUnlocked
    ]
    
    // Print the query for debugging
    print("SWIFT DEBUG: Add query: \(addQuery)")
    
    let status = SecItemAdd(addQuery as CFDictionary, nil)
    print("SWIFT DEBUG: Add item status: \(status)")
    
    if status == errSecSuccess {
        print("SWIFT DEBUG: Successfully added token")
        return Status.success.rawValue
    } else {
        print("SWIFT DEBUG: Failed to add token, error: \(status)")
        return Status.failed.rawValue
    }
}

@_cdecl("get_token")
public func getToken(_ keyPtr: UnsafePointer<CChar>) -> UnsafePointer<CChar>? {
    let keyString = String(cString: keyPtr)
    
    let query: [CFString: Any] = [
        kSecClass: kSecClassGenericPassword,
        kSecAttrAccount: keyString,
        kSecAttrService: "com.thenicekat.srs",
        kSecReturnData: true,
        kSecMatchLimit: kSecMatchLimitOne
    ]
    
    var item: CFTypeRef?
    let status = SecItemCopyMatching(query as CFDictionary, &item)
    
    if status == errSecSuccess, let data = item as? Data, let tokenString = String(data: data, encoding: .utf8) {

        if let result = strdup(tokenString) {
    
            return UnsafePointer(result)
        } else {
    
        }
    } else {

    }
    
    return nil
}

@_cdecl("list_tokens")
public func listTokens() -> UnsafePointer<CChar>? {
    print("SWIFT DEBUG: listTokens called")
    
    let query: [CFString: Any] = [
        kSecClass: kSecClassGenericPassword,
        kSecAttrService: "com.thenicekat.srs",
        kSecReturnAttributes: true,
        kSecMatchLimit: kSecMatchLimitAll
    ]
    
    var items: CFTypeRef?
    let status = SecItemCopyMatching(query as CFDictionary, &items)
    print("SWIFT DEBUG: SecItemCopyMatching status: \(status)")
    
    if status == errSecSuccess, let itemsArray = items as? [[String: Any]] {
        print("SWIFT DEBUG: Found \(itemsArray.count) items in keychain")
        
        let keys = itemsArray.compactMap { $0[kSecAttrAccount as String] as? String }
        print("SWIFT DEBUG: Keys: \(keys)")
        
        if let jsonData = try? JSONSerialization.data(withJSONObject: keys, options: []),
           let jsonString = String(data: jsonData, encoding: .utf8) {
            print("SWIFT DEBUG: JSON string: '\(jsonString)'")
            
            // Ensure we're returning a valid C string
            if let result = strdup(jsonString) {
                print("SWIFT DEBUG: Returning JSON string")
                return UnsafePointer(result)
            } else {
                print("SWIFT DEBUG: Failed to create C string with strdup")
            }
        } else {
            print("SWIFT DEBUG: Failed to serialize to JSON")
        }
    } else {
        print("SWIFT DEBUG: No items found or error occurred, status: \(status)")
        if status == errSecItemNotFound {
            print("SWIFT DEBUG: No items found in keychain")
        } else {
            print("SWIFT DEBUG: Error accessing keychain: \(status)")
        }
    }
    
    print("SWIFT DEBUG: Returning empty array")
    let emptyArray = "[]"
    
    // Create a new C string with the empty array
    if let result = strdup(emptyArray) {
        print("SWIFT DEBUG: Empty array C string created")
        return UnsafePointer(result)
    }
    
    print("SWIFT DEBUG: Failed to create empty array C string")
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
