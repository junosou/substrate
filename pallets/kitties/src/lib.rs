#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// 了解更多关于FRAME和Substrate FRAME 仓库的核心内容:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

//将package加进来
mod migrations;


#[frame_support::pallet]
//pallet 划分traits来实现它的功能 需要引入trait，定义在support里面
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::StorageVersion};
	use frame_system::pallet_prelude::*;

	use frame_support::{
		traits::{Currency, ExistenceRequirement, Randomness},
		PalletId,
	};
	use sp_runtime::traits::AccountIdConversion;

	use sp_io::hashing::blake2_128;

	pub type KittyId = u32;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	
	//增加currency操作，currency作为一个traits特征,在Currency中会定义Balance的类型，,使用这个Balance或者代币单位,需要这个类型的定义,有了Price可以创建一个Kitty执行reserve操作  

	#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
	// pub struct Kitty(pub [u8; 16]);
	////dna可读性差，加上名字，做一个新的数据结构
	pub struct Kitty {
		pub dna: [u8; 16], 
		pub name: [u8; 8],
		//第一个 dna，2.名字
	}
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);
	//更新，创建STORAGE_VERSION常量，改一下版本号

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	//增加一个attribude
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	//	通过指定依赖的参数和类型来配置pallet。

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// 因为这个Pallet会发出事件,所以它依赖于运行时对事件的定义。
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId>;
		#[pallet::constant]
		type KittyPrice: Get<BalanceOf<Self>>;
		//价格 定义kitty常量
		type PalletId: Get<PalletId>;
		// 定义palletid，可以转换成装户
	}

	// pallet的运行时存储项。 
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_parents)]
	pub type KittyParents<T: Config> =
		StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_on_sale)]
	pub type KittyOnSale<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, (), OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	//实现 pallet 事件
	pub enum Event<T: Config> {
		KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyBred { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyTransferred { who: T::AccountId, recipient: T::AccountId, kitty_id: KittyId },
		KittyOnSale { who: T::AccountId, kitty_id: KittyId },
		KittyBought { who: T::AccountId, kitty_id: KittyId },
	}
	//错误处理 substrate 使用 #[pallet::error] 定义错误，我们在 Error 中添加如下代码：
	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		SamedKittyId,
		NotOwner,
		AlreadyOnSale,
		//新的error类型
		NoOwner,
		AlreadyOwned,
		NotOnSale,
	}

	//调用hooks方法，内部调用，方法名字是on_runtime_upgrade，最后返回weight，添加好把它加进来
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			// migrations::v1::migrate::<T>()
			migrations::v2::migrate::<T>()
			////函数调用放在hook，直接引用版本
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		pub fn create(origin: OriginFor<T>, name: [u8; 8]) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			let kitty_id = Self::get_kitty_id()?;
			let dna = Self::random_value(&who);
			let kitty = Kitty { dna, name };

			let price = T::KittyPrice::get();
			//根据type get的方法取得price
			// T::Currency::reserve(&who, price)?;
			T::Currency::transfer(&who, &Self::get_account_id(), price, ExistenceRequirement::KeepAlive)?;

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);

			// Emit an event.
			Self::deposit_event(Event::KittyCreated { who, kitty_id, kitty });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({0})]
		//繁殖小猫的函数比较简单，代码如下：
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: KittyId,
			kitty_id_2: KittyId,
			name: [u8; 8]
			//数据结构调整
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SamedKittyId);

			ensure!(Kitties::<T>::contains_key(kitty_id_1), Error::<T>::InvalidKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id_2), Error::<T>::InvalidKittyId);

			let kitty_1 = Kitties::<T>::get(kitty_id_1).expect("We checked it exists");
			let kitty_2 = Kitties::<T>::get(kitty_id_2).expect("We checked it exists");

			let kitty_id = Self::get_kitty_id()?;

			let selector = Self::random_value(&who);
			let mut dna = [0u8; 16];
			for i in 0..kitty_1.dna.len() {
				dna[i] = selector[i] & kitty_1.dna[i] | !selector[i] & kitty_2.dna[i];
			}

			let kitty = Kitty{dna, name};

			let price = T::KittyPrice::get();
			// T::Currency::reserve(&who, price)?; 
			
			//和create一样，需要调用这个方法去transfer你的token到pallet account里面，调用方法一样
			T::Currency::transfer(
				&who,
				&Self::get_account_id(),
				price,
				ExistenceRequirement::KeepAlive,
			)?;

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			KittyParents::<T>::insert(kitty_id, (kitty_id_1, kitty_id_2));

			Self::deposit_event(Event::KittyBred { who, kitty_id, kitty });

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight({0})]
		//交易 kitty,transfer 函数的代码如下：
		pub fn transfer(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			kitty_id: KittyId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);
			ensure!(KittyOwner::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);

			let owner = KittyOwner::<T>::get(kitty_id).unwrap();
			ensure!(owner == who, Error::<T>::NotOwner);
			KittyOwner::<T>::insert(kitty_id, &recipient);

			Self::deposit_event(Event::KittyTransferred { who, recipient, kitty_id });

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight({0})] 
		//对kitty可以实现买卖，新增定义方法 sale，有这个方法做标示
		pub fn sale(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);
			//是否在？
			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::NoOwner)?;
			ensure!(owner == who, Error::<T>::NotOwner);
			ensure!(!KittyOnSale::<T>::contains_key(kitty_id), Error::<T>::AlreadyOnSale);
			//错误类型 AlreadyOnSale
			KittyOnSale::<T>::insert(kitty_id, ());
			//链上的状态表示，增加存储
			Self::deposit_event(Event::KittyOnSale { who, kitty_id });
			//  抛出KittyOnSale
			Ok(())
		}
//下一个方法，另一个account实际买kitty
		#[pallet::call_index(4)]
		#[pallet::weight({0})]

//检查 kitty 是否可以购买, 购买 kitty 时我们需要从两个方面确认可以购买：1、这只 kitty 的状态是要等待购买；
//2、当前这只 kitty 是否在用户的预算之类，并且用户有足够的余额
//支付 支付时直接使用 Currency::transfer 进行，完了后转移 kitty 的所有权到买家，最后发出事件。

		pub fn buy(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);
			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::NoOwner)?;
			ensure!(owner != who, Error::<T>::AlreadyOwned);
			ensure!(KittyOnSale::<T>::contains_key(kitty_id), Error::<T>::NotOnSale);
			//基本判断
			//得到价格
			let price = T::KittyPrice::get();
			//	调用方法一样，从旧的买家转到新的买家
			T::Currency::transfer(&who, &owner, price, ExistenceRequirement::KeepAlive)?;
			// T::Currency::reserve(&who, &Self::get_account_id, price, 如果质押需要调用currency的 reservce 方法
			// ExistenceRequirement::KeepAlive)?; T::Currency::unreserve(&owner, price);

			KittyOwner::<T>::insert(kitty_id, &who);
			KittyOnSale::<T>::remove(kitty_id);
			// 状态转换
			Self::deposit_event(Event::KittyBought { who, kitty_id });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_kitty_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|next_id| -> Result<KittyId, DispatchError> {
				let current_id = *next_id;
				*next_id = next_id.checked_add(1).ok_or(Error::<T>::InvalidKittyId)?;
				Ok(current_id)
			})
		}

		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				frame_system::Pallet::<T>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}

		fn get_account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		//get方法得到值 添加辅助方法，引入账号
		}
	}
}
