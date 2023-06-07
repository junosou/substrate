use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	migration::storage_key_iter, storage::StoragePrefixedMap,
	traits::GetStorageVersion, weights::Weight, Blake2_128Concat,
};
use scale_info::TypeInfo;

use crate::{Config, Kitties, KittyId, Pallet};

//拷贝一下升级前数据结构，命名为OldKitty
#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct OldKitty(pub [u8; 16]);

//migrate方法，需要config 返回值的类型
pub fn migrate<T: Config>() -> Weight {
	//需要得到两个版本号，老得版本和设置上去的版本号，变量current_version， 类似方法current_storage_version
	let on_chain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::current_storage_version();
	
	//做一下判断
	if on_chain_version != 0 {
		return Weight::zero()
	}

	//！=1 退出
	if current_version != 1 {
		return Weight::zero()
	}
	//只能 0-1 升级
	
	//参考substrate文档，两个最前面的prefix
	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();
// 用到iter方法，Blake2_128Concat 是hash函数，可以做个循环，drain（老的数据不要），相当于把kitty数据春初所有map取出来，用drain去掉，老得数据结合新的数据
	for (index, kitty) in storage_key_iter::<KittyId, OldKitty, Blake2_128Concat>(module, item).drain() {
		let new_kitty = crate::Kitty {
			dna: kitty.0,
			name: *b"None",
		};
		Kitties::<T>::insert(index, new_kitty);
	}

	todo!()
}
