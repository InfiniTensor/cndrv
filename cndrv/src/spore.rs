//! 资源-孢子这一对抽象保证了基于上下文的驱动设计的安全性。
//!
//! 由于加速硬件与 CPU 是异构异步的，驱动引入了硬件上下文的概念，代表某个加速硬件上的一系列资源。
//! 所有申请、释放加速器资源（包括存储空间、算力、调度能力等）的操作都需要在某个硬件上下文中进行。
//! 同一个资源只能释放回申请它的上下文，不能串台。
//!
//! 在 ABI 中，这些限制需要程序员人为保证，对于包含多个上下文的应用程序来说，上下文管理非常难。
//! 因此，cndrv crate 提出资源-孢子这一对抽象封装上下文上的资源，并借助编译器规则和运行时检查保证安全性。
//!
//! 资源-孢子的抽象由两个 trait 组成：
//!
//! - [`ContextResource`] 即上下文资源。
//!   这个 trait 将资源的生命周期与上下文绑定，以保证资源的申请和释放在同一个上下文中完成。
//!   处于上下文资源状态的资源对象可以参与相应的功能。例如，处于上下文资源状态的存储空间可以读写。
//!   但是业务逻辑中，不可避免地会出现需要暂时切换当前上下文而不释放资源的情况，
//!   因此资源提供 [`sporulate`](ContextResource::sporulate) 方法将资源转换为孢子。
//! - [`ContextSpore`] 即上下文孢子。
//!   这个 trait 移除上下文资源的生命周期约束，从而允许在上下文被换出时暂时保持资源存在。
//!   但处于上下文孢子状态的资源通常不再允许参与相应的功能。
//!   只有当申请这些资源的上下文被换回，并在上下文上将孢子恢复为资源后才能继续发挥作用。
//!   上下文孢子提供 [`sprout`](ContextSpore::sprout) 方法将孢子转换为资源，
//!   以及 [`sprout_ref`](ContextSpore::sprout_ref) 和 [`sprout_mut`](ContextSpore::sprout_mut) 方法获取资源的不可变和可变引用。
//!   这些方法将引入运行时检查以保证孢子在正确的上下文上复原。
//!
//! [`spore_convention`] 宏提供了基于 [`RawContainer`] 类型实现的资源-孢子对的标准版本。

use crate::{bindings::CNcontext, CurrentCtx};

pub trait ContextResource<'ctx> {
    type Spore: ContextSpore<Resource<'ctx> = Self>;

    fn sporulate(self) -> Self::Spore;
}

pub trait ContextSpore: 'static + Send + Sync {
    type Resource<'ctx>: ContextResource<'ctx, Spore = Self>;

    fn sprout(self, ctx: &CurrentCtx) -> Self::Resource<'_>;
    fn sprout_ref<'ctx>(&'ctx self, ctx: &'ctx CurrentCtx) -> &Self::Resource<'_>;
    fn sprout_mut<'ctx>(&'ctx mut self, ctx: &'ctx CurrentCtx) -> &mut Self::Resource<'_>;
}

#[macro_export]
macro_rules! spore_convention {
    ($spore:ty) => {
        unsafe impl Send for $spore {}
        unsafe impl Sync for $spore {}
        impl Drop for $spore {
            #[inline]
            fn drop(&mut self) {
                unreachable!("Never drop ContextSpore");
            }
        }
    };
}

pub struct RawContainer<T: Unpin> {
    pub ctx: CNcontext,
    pub raw: T,
}

#[macro_export]
macro_rules! impl_spore {
    ($resource:ident and $spore:ident by $kernel:ty) => {
        #[repr(transparent)]
        pub struct $resource<'ctx>(
            $crate::RawContainer<$kernel>,
            std::marker::PhantomData<&'ctx ()>,
        );

        impl<'ctx> $resource<'ctx> {
            #[inline]
            pub fn ctx(&self) -> &$crate::CurrentCtx {
                unsafe { $crate::CurrentCtx::from_raw(&self.0.ctx) }
            }
        }

        #[repr(transparent)]
        pub struct $spore($crate::RawContainer<$kernel>);

        $crate::spore_convention!($spore);

        impl $crate::ContextSpore for $spore {
            type Resource<'ctx> = $resource<'ctx>;

            #[inline]
            fn sprout(self, ctx: &$crate::CurrentCtx) -> Self::Resource<'_> {
                assert_eq!(self.0.ctx, unsafe {
                    <$crate::CurrentCtx as $crate::AsRaw>::as_raw(ctx)
                });
                // SAFETY: `transmute_copy` + `forget` 是手工实现移动语义。
                // `RawContainer`` 具有 `Unpin` 保证它的安全性。
                let ans = unsafe { std::mem::transmute_copy(&self.0) };
                std::mem::forget(self);
                ans
            }

            #[inline]
            fn sprout_ref<'ctx>(&'ctx self, ctx: &'ctx $crate::CurrentCtx) -> &Self::Resource<'_> {
                assert_eq!(self.0.ctx, unsafe {
                    <$crate::CurrentCtx as $crate::AsRaw>::as_raw(ctx)
                });
                // SAFETY: 资源以引用的形式返回，因此在使用完成后不会释放。
                unsafe { std::mem::transmute(&self.0) }
            }

            #[inline]
            fn sprout_mut<'ctx>(
                &'ctx mut self,
                ctx: &'ctx $crate::CurrentCtx,
            ) -> &mut Self::Resource<'_> {
                assert_eq!(self.0.ctx, unsafe {
                    <$crate::CurrentCtx as $crate::AsRaw>::as_raw(ctx)
                });
                // SAFETY: 资源以可变引用的形式返回，因此在使用完成后不会释放。
                unsafe { std::mem::transmute(&mut self.0) }
            }
        }

        impl<'ctx> $crate::ContextResource<'ctx> for $resource<'ctx> {
            type Spore = $spore;

            #[inline]
            fn sporulate(self) -> Self::Spore {
                // SAFETY: `transmute_copy` + `forget` 是手工实现移动语义。
                // `RawContainer`` 具有 `Unpin` 保证它的安全性。
                let s = unsafe { std::mem::transmute_copy(&self.0) };
                std::mem::forget(self);
                $spore(s)
            }
        }
    };
}
