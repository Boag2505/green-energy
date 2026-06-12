#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

// 1. Định nghĩa các cấu trúc dữ liệu lưu trữ trên Blockchain
#[contracttype]
pub enum DataKey {
    Admin,            // Lưu địa chỉ ví của Project Owner (Người quản lý)
    TotalShares,      // Lưu tổng cung Token (Ví dụ: 10,000 SLR)
    TotalDividends,   // Lưu tổng số USDC cổ tức đã nạp vào từ trước đến nay
    Balance(Address), // Lưu số dư Token SLR của từng ví nhà đầu tư
    Claimed(Address), // Lưu tổng số USDC mà một ví cụ thể đã rút (để tránh rút lặp)
}

#[contract]
pub struct GreenEnergyRWA;

#[contractimpl]
impl GreenEnergyRWA {
    
    // ==========================================
    // HÀM 1: KHỞI TẠO DỰ ÁN
    // ==========================================
    pub fn initialize(env: Env, admin: Address, total_shares: i128) {
        admin.require_auth(); // Bắt buộc ví admin phải ký xác nhận
        
        // Lưu các thông số ban đầu vào storage của Smart Contract
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TotalShares, &total_shares);
        env.storage().instance().set(&DataKey::TotalDividends, &0_i128);
    }

    // ==========================================
    // HÀM 2: PHÂN BỔ TOKEN (GIẢ LẬP MUA BÁN)
    // ==========================================
    pub fn mint_shares(env: Env, admin: Address, to: Address, amount: i128) {
        admin.require_auth();
        
        // Kiểm tra đúng admin mới được phép phân bổ token
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        assert!(admin == stored_admin, "Loi: Chi admin moi duoc phan bo token!");

        // Cộng token vào ví của nhà đầu tư
        let mut balance: i128 = env.storage().instance().get(&DataKey::Balance(to.clone())).unwrap_or(0);
        balance += amount;
        env.storage().instance().set(&DataKey::Balance(to), &balance);
    }

    // ==========================================
    // HÀM 3: ADMIN NẠP TIỀN ĐIỆN HÀNG THÁNG
    // ==========================================
    pub fn deposit_yield(env: Env, admin: Address, yield_amount: i128) {
        admin.require_auth();
        
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        assert!(admin == stored_admin, "Loi: Chi admin moi duoc nap co tuc!");

        // Cộng dồn doanh thu tiền điện vào tổng quỹ
        let mut total_dividends: i128 = env.storage().instance().get(&DataKey::TotalDividends).unwrap_or(0);
        total_dividends += yield_amount;
        env.storage().instance().set(&DataKey::TotalDividends, &total_dividends);
    }

    // ==========================================
    // HÀM 4: NHÀ ĐẦU TƯ RÚT CỔ TỨC (RWA CORE LOGIC)
    // ==========================================
    pub fn claim_yield(env: Env, user: Address) -> i128 {
        user.require_auth(); // Bắt buộc user ký giao dịch để lấy tiền
        
        // Lấy số lượng token user đang giữ
        let shares: i128 = env.storage().instance().get(&DataKey::Balance(user.clone())).unwrap_or(0);
        assert!(shares > 0, "Loi: Ban khong so huu Token nao!");

        let total_shares: i128 = env.storage().instance().get(&DataKey::TotalShares).unwrap();
        let total_dividends: i128 = env.storage().instance().get(&DataKey::TotalDividends).unwrap();
        
        // Lấy số tiền user đã rút trong quá khứ
        let claimed: i128 = env.storage().instance().get(&DataKey::Claimed(user.clone())).unwrap_or(0);

        // THUẬT TOÁN MINH BẠCH: Tính số tiền thực tế user được nhận ngay lúc này
        // (Số cổ phần đang giữ * Tổng doanh thu / Tổng cổ phần) - Số đã rút
        let total_earned = (shares * total_dividends) / total_shares;
        let owe = total_earned - claimed;

        assert!(owe > 0, "Loi: Khong co co tuc moi de rut!");

        // Lưu lại lịch sử để user không thể ấn claim 2 lần cho cùng một số tiền
        let new_claimed = claimed + owe;
        env.storage().instance().set(&DataKey::Claimed(user.clone()), &new_claimed);

        // (Ở dự án thực tế, bạn sẽ thêm một lệnh transfer USDC từ contract về ví user ở đây)
        
        // Trả về kết quả hiển thị cho UI
        owe 
    }
}
