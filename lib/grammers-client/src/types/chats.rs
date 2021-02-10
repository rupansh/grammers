// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::types::{Chat, Role, User};
use crate::ClientHandle;
use grammers_mtproto::mtp::RpcError;
use grammers_mtsender::InvocationError;
use tokio::pin;
use grammers_tl_types as tl;
use pin_project::pin_project;
use std::{
    future::Future,
    task::{Context, Poll},
    pin::Pin,
    mem::drop,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[pin_project]
struct FutWrap<T, F: Future<Output = Result<T, InvocationError>> + Send + 'static > {
    #[pin]
    fut: F
}

impl<T, F: Future<Output = Result<T, InvocationError>> + Send + 'static > FutWrap<T, F> {
    fn new(f: F) -> Self {
        FutWrap { fut: f }
    }
}

impl<T, F> Future for FutWrap<T, F>
    where F: Future<Output = Result<T, InvocationError>> + Send + 'static 
{
    type Output = Result<T, InvocationError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().fut.poll(cx)
    }
}

#[derive(Clone)]
pub struct AdminRightsBuilderInner {
    client: ClientHandle,
    chat: Chat,
    user: tl::enums::InputUser,
    rights: tl::types::ChatAdminRights,
    rank: String,
}

/// Builder for editing the administrator rights of a user in a specific chat.
///
/// Use [`ClientHandle::set_admin_rights`] to retrieve an instance of this type.
#[pin_project]
pub struct AdminRightsBuilder<F, Fnc> 
where
    F: Future<Output = Result<(), InvocationError>> + Send + 'static ,
    Fnc: Fn(AdminRightsBuilderInner) -> F 
{
    inner: AdminRightsBuilderInner,
    f: Fnc,
    #[pin]
    fut: Option<FutWrap<(), F>>
}

impl<F, Fnc> Future for AdminRightsBuilder<F, Fnc>
    where F: Future<Output = Result<(), InvocationError>> + Send + 'static ,
          Fnc: Fn(AdminRightsBuilderInner) -> F 
{
    type Output = Result<(), InvocationError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut proj = self.project();
        if proj.fut.is_none() {
            let func = proj.f;
            let inner = proj.inner.clone();
            let ok = Some(FutWrap::<(), F>::new(func(inner)));
            pin!(ok);
            proj.fut = ok;
            Future::poll(proj.fut.as_pin_mut().unwrap(), cx)
        } else {
            Future::poll(proj.fut.as_pin_mut().unwrap(), cx)
        }
    }
}


pub async fn invoke_wrap(s: AdminRightsBuilderInner) -> Result<(), InvocationError> {
    let mut s = s;
    if let Some(chan) = s.chat.to_input_channel() {
        s.client
            .invoke(&tl::functions::channels::EditAdmin {
                channel: chan,
                user_id: s.user.clone(),
                admin_rights: tl::enums::ChatAdminRights::Rights(s.rights.clone()),
                rank: s.rank.clone(),
            })
            .await
            .map(drop)
    } else if let Some(id) = s.chat.to_chat_id() {
        let promote = if s.rights.anonymous
            || s.rights.change_info
            || s.rights.post_messages
            || s.rights.edit_messages
            || s.rights.delete_messages
            || s.rights.ban_users
            || s.rights.invite_users
            || s.rights.pin_messages
            || s.rights.add_admins
            || s.rights.manage_call
        {
            true
        } else {
            false
        };
        s.client
            .invoke(&tl::functions::messages::EditChatAdmin {
                chat_id: id,
                user_id: s.user.clone(),
                is_admin: promote,
            })
            .await
            .map(drop)
    } else {
        Err(InvocationError::Rpc(RpcError {
            code: 400,
            name: "PEER_ID_INVALID".to_string(),
            value: None,
        }))
    }
}

/*pub(crate) async fn invoke_wrap<F, Fnc>(s: &mut AdminRightsBuilder<F, Fnc>) -> Result<(), InvocationError>
    where F: Future<Output = Result<(), InvocationError>> + Send + 'static,
          Fnc: Fn(&mut AdminRightsBuilder<F, Fnc>) -> F
{
    return s.invoke().await
}*/

pub trait AdminBuilderImpl<'a>: 'a {
    //type Output;


//    fn load_current(&'a mut self) -> Self::Output;

    fn anonymous(self, val: bool) -> Self;
    fn manage_call(self, val: bool) -> Self;
    fn change_info(self, val: bool) -> Self;
    fn post_messages(self, val: bool) -> Self;
    fn edit_messages(self, val: bool) -> Self;
    fn delete_messages(self, val: bool) -> Self;
    fn ban_users(self, val: bool) -> Self;
    fn invite_users(self, val: bool) -> Self;
    fn pin_messages(self, val: bool) -> Self;
    fn add_admins(self, val: bool) -> Self;
    fn rank<S: Into<String>>(self, val: S) -> Self;
}

impl<'a, F, Fnc> AdminBuilderImpl<'a> for AdminRightsBuilder<F, Fnc>
where
    F: Future<Output = Result<(), InvocationError>> + Send + 'static ,
    Fnc: (Fn(AdminRightsBuilderInner) -> F) + 'a ,
{
//    type Output = FutWrap<Result<&'a mut Self, InvocationError>, impl Future<Output = Result<&'a mut Self, InvocationError>>>;

    /*fn load_current(&mut self) -> Self::Output {
        let c = async { if let Some(chan) = self.inner.chat.to_input_channel() {
            let tl::enums::channels::ChannelParticipant::Participant(user) = self
                .inner.client
                .invoke(&tl::functions::channels::GetParticipant {
                    channel: chan,
                    user_id: self.inner.user.clone(),
                })
                .await?;
            match user.participant {
                tl::enums::ChannelParticipant::Creator(c) => {
                    self.inner.rights = c.admin_rights.into();
                    self.inner.rank = c.rank.unwrap_or_else(String::new);
                }
                tl::enums::ChannelParticipant::Admin(a) => {
                    self.inner.rights = a.admin_rights.into();
                    self.inner.rank = a.rank.unwrap_or_else(String::new);
                }
                _ => (),
            }
        } else if matches! {self.inner.chat, Chat::Group(_)} {
            let uid = match &self.inner.user {
                tl::enums::InputUser::User(u) => u.user_id,
                tl::enums::InputUser::FromMessage(u) => u.user_id,
                _ => {
                    return Err(InvocationError::Rpc(RpcError {
                        code: 400,
                        name: "PEER_ID_INVALID".to_string(),
                        value: None,
                    }))
                }
            };

            let mut participants = self.inner.client.iter_participants(&self.inner.chat);
            while let Some(participant) = participants.next().await? {
                if matches!(participant.role, Role::Creator(_) | Role::Admin(_))
                    && participant.user.id() == uid
                {
                    self.inner.rights = tl::types::ChatAdminRights {
                        change_info: true,
                        post_messages: true,
                        edit_messages: false,
                        delete_messages: true,
                        ban_users: true,
                        invite_users: true,
                        pin_messages: true,
                        add_admins: true,
                        anonymous: false,
                        manage_call: true,
                    };
                    break;
                }
            }
        }

        Ok(self) 
        };
        return c;
    }*/

    /// Whether the user will remain anonymous when sending messages.
    ///
    /// The sender of the anonymous messages becomes the group itself.
    ///
    /// Note that other people in the channel may be able to identify the anonymous admin by its
    /// custom rank, so additional care is needed when using both anonymous and custom ranks.
    ///
    /// For example, if multiple anonymous admins share the same title, users won't be able to
    /// distinguish them.
    fn anonymous(mut self, val: bool) -> Self {
        self.inner.rights.anonymous = val;
        self
    }

    /// Whether the user is able to manage calls in the group.
    fn manage_call(mut self, val: bool) -> Self {
        self.inner.rights.manage_call = val;
        self
    }

    /// Whether the user is able to change information about the chat such as group description or
    /// not.
    fn change_info(mut self, val: bool) -> Self {
        self.inner.rights.change_info = val;
        self
    }

    /// Whether the user will be able to post in the channel. This will only work in broadcast
    /// channels, not groups.
    fn post_messages(mut self, val: bool) -> Self {
        self.inner.rights.post_messages = val;
        self
    }

    /// Whether the user will be able to edit messages in the channel. This will only work in
    /// broadcast channels, not groups.
    fn edit_messages(mut self, val: bool) -> Self {
        self.inner.rights.edit_messages = val;
        self
    }

    /// Whether the user will be able to delete messages. This includes messages from others.
    fn delete_messages(mut self, val: bool) -> Self {
        self.inner.rights.delete_messages = val;
        self
    }

    /// Whether the user will be able to edit the restrictions of other users. This effectively
    /// lets the administrator ban (or kick) people.
    fn ban_users(mut self, val: bool) -> Self {
        self.inner.rights.ban_users = val;
        self
    }

    /// Whether the user will be able to invite other users.
    fn invite_users(mut self, val: bool) -> Self {
        self.inner.rights.invite_users = val;
        self
    }

    /// Whether the user will be able to pin messages.
    fn pin_messages(mut self, val: bool) -> Self {
        self.inner.rights.pin_messages = val;
        self
    }

    /// Whether the user will be able to add other administrators with the same or less
    /// permissions than the user itself.
    fn add_admins(mut self, val: bool) -> Self {
        self.inner.rights.add_admins = val;
        self
    }

    /// The custom rank  (also known as "admin title" or "badge") to show for this administrator.
    ///
    /// This text will be shown instead of the "admin" badge.
    ///
    /// When left unspecified or empty, the default localized "admin" badge will be shown.
    fn rank<S: Into<String>>(mut self, val: S) -> Self {
        self.inner.rank = val.into();
        self
    }

}

impl<F, Fnc> AdminRightsBuilder<F, Fnc> 
where
    F: Future<Output = Result<(), InvocationError>> + Send + 'static ,
    Fnc: Fn(AdminRightsBuilderInner) -> F 
{
    pub(crate) fn new(client: ClientHandle, chat: &Chat, user: &User, func: Fnc) -> Self {
        AdminRightsBuilder {
            inner: AdminRightsBuilderInner { client,
            chat: chat.clone(),
            user: user.to_input(),
            rank: "".into(),
            rights: tl::types::ChatAdminRights {
                anonymous: false,
                change_info: false,
                post_messages: false,
                edit_messages: false,
                delete_messages: false,
                ban_users: false,
                invite_users: false,
                pin_messages: false,
                add_admins: false,
                manage_call: false,
            }},
            f: func,
            fut: None
        }
    }

    /// Load the current rights of the user. This lets you trivially grant or take away specific
    /// permissions without changing any of the previous ones.
    pub async fn load_current(&mut self) -> Result<&mut Self, InvocationError> {
        if let Some(chan) = self.inner.chat.to_input_channel() {
            let tl::enums::channels::ChannelParticipant::Participant(user) = self
                .inner.client
                .invoke(&tl::functions::channels::GetParticipant {
                    channel: chan,
                    user_id: self.inner.user.clone(),
                })
                .await?;
            match user.participant {
                tl::enums::ChannelParticipant::Creator(c) => {
                    self.inner.rights = c.admin_rights.into();
                    self.inner.rank = c.rank.unwrap_or_else(String::new);
                }
                tl::enums::ChannelParticipant::Admin(a) => {
                    self.inner.rights = a.admin_rights.into();
                    self.inner.rank = a.rank.unwrap_or_else(String::new);
                }
                _ => (),
            }
        } else if matches! {self.inner.chat, Chat::Group(_)} {
            let uid = match &self.inner.user {
                tl::enums::InputUser::User(u) => u.user_id,
                tl::enums::InputUser::FromMessage(u) => u.user_id,
                _ => {
                    return Err(InvocationError::Rpc(RpcError {
                        code: 400,
                        name: "PEER_ID_INVALID".to_string(),
                        value: None,
                    }))
                }
            };

            let mut participants = self.inner.client.iter_participants(&self.inner.chat);
            while let Some(participant) = participants.next().await? {
                if matches!(participant.role, Role::Creator(_) | Role::Admin(_))
                    && participant.user.id() == uid
                {
                    self.inner.rights = tl::types::ChatAdminRights {
                        change_info: true,
                        post_messages: true,
                        edit_messages: false,
                        delete_messages: true,
                        ban_users: true,
                        invite_users: true,
                        pin_messages: true,
                        add_admins: true,
                        anonymous: false,
                        manage_call: true,
                    };
                    break;
                }
            }
        }

        Ok(self)
    }
}

/// Builder for editing the rights of a non-admin user in a specific chat.
///
/// Certain groups (small group chats) only allow banning (disallow `view_messages`). Trying to
/// disallow other permissions in these groups will fail.
///
/// Use [`ClientHandle::set_banned_rights`] to retrieve an instance of this type.
pub struct BannedRightsBuilder {
    client: ClientHandle,
    chat: Chat,
    user: tl::enums::InputUser,
    rights: tl::types::ChatBannedRights,
}

impl BannedRightsBuilder {
    pub(crate) fn new(client: ClientHandle, chat: &Chat, user: &User) -> Self {
        Self {
            client,
            chat: chat.clone(),
            user: user.to_input(),
            rights: tl::types::ChatBannedRights {
                view_messages: false,
                send_messages: false,
                send_media: false,
                send_stickers: false,
                send_gifs: false,
                send_games: false,
                send_inline: false,
                embed_links: false,
                send_polls: false,
                change_info: false,
                invite_users: false,
                pin_messages: false,
                until_date: 0,
            },
        }
    }

    /// Load the current rights of the user. This lets you trivially grant or take away specific
    /// permissions without changing any of the previous ones.
    pub async fn load_current(&mut self) -> Result<&mut Self, InvocationError> {
        if let Some(chan) = self.chat.to_input_channel() {
            let tl::enums::channels::ChannelParticipant::Participant(user) = self
                .client
                .invoke(&tl::functions::channels::GetParticipant {
                    channel: chan,
                    user_id: self.user.clone(),
                })
                .await?;
            match user.participant {
                tl::enums::ChannelParticipant::Banned(u) => {
                    self.rights = u.banned_rights.into();
                }
                _ => (),
            }
        }

        Ok(self)
    }

    /// Whether the user is able to view messages or not. Forbidding someone from viewing messages
    /// effectively bans (kicks) them.
    pub fn view_messages(&mut self, val: bool) -> &mut Self {
        // `true` indicates "take away", but in the builder it makes more sense that `false` means
        // "they won't have this permission". All methods perform this negation for that reason.
        self.rights.view_messages = !val;
        self
    }

    /// Whether the user is able to send messages or not. The user will remain in the chat, and
    /// can still read the conversation.
    pub fn send_messages(&mut self, val: bool) -> &mut Self {
        self.rights.send_messages = !val;
        self
    }

    /// Whether the user is able to send any form of media or not, such as photos or voice notes.
    pub fn send_media(&mut self, val: bool) -> &mut Self {
        self.rights.send_media = !val;
        self
    }

    /// Whether the user is able to send stickers or not.
    pub fn send_stickers(&mut self, val: bool) -> &mut Self {
        self.rights.send_stickers = !val;
        self
    }

    /// Whether the user is able to send animated gifs or not.
    pub fn send_gifs(&mut self, val: bool) -> &mut Self {
        self.rights.send_gifs = !val;
        self
    }

    /// Whether the user is able to send games or not.
    pub fn send_games(&mut self, val: bool) -> &mut Self {
        self.rights.send_games = !val;
        self
    }

    /// Whether the user is able to use inline bots or not.
    pub fn send_inline(&mut self, val: bool) -> &mut Self {
        self.rights.send_inline = !val;
        self
    }

    /// Whether the user is able to enable the link preview in the messages they send.
    ///
    /// Note that the user will still be able to send messages with links if this permission is
    /// taken away from the user, but these links won't display a link preview.
    pub fn embed_link_previews(&mut self, val: bool) -> &mut Self {
        self.rights.embed_links = !val;
        self
    }

    /// Whether the user is able to send polls or not.
    pub fn send_polls(&mut self, val: bool) -> &mut Self {
        self.rights.send_polls = !val;
        self
    }

    /// Whether the user is able to change information about the chat such as group description or
    /// not.
    pub fn change_info(&mut self, val: bool) -> &mut Self {
        self.rights.change_info = !val;
        self
    }

    /// Whether the user is able to invite other users or not.
    pub fn invite_users(&mut self, val: bool) -> &mut Self {
        self.rights.invite_users = !val;
        self
    }

    /// Whether the user is able to pin messages or not.
    pub fn pin_messages(&mut self, val: bool) -> &mut Self {
        self.rights.pin_messages = !val;
        self
    }

    /// Apply the restrictions until the given epoch time.
    ///
    /// Note that this is absolute time (i.e current time is not added).
    ///
    /// By default, the restriction is permanent.
    pub fn until(&mut self, val: i32) -> &mut Self {
        // TODO this should take a date, not int
        self.rights.until_date = val;
        self
    }

    /// Apply the restriction for a given duration.
    pub fn duration(&mut self, val: Duration) -> &mut Self {
        // TODO this should account for the server time instead (via sender's offset)
        self.rights.until_date = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before epoch")
            .as_secs() as i32
            + val.as_secs() as i32;

        self
    }

    /// Perform the call.
    pub async fn invoke(&mut self) -> Result<(), InvocationError> {
        if let Some(chan) = self.chat.to_input_channel() {
            self.client
                .invoke(&tl::functions::channels::EditBanned {
                    channel: chan,
                    user_id: self.user.clone(),
                    banned_rights: tl::enums::ChatBannedRights::Rights(self.rights.clone()),
                })
                .await
                .map(drop)
        } else if let Some(id) = self.chat.to_chat_id() {
            if self.rights.view_messages {
                self.client
                    .invoke(&tl::functions::messages::DeleteChatUser {
                        chat_id: id,
                        user_id: self.user.clone(),
                    })
                    .await
                    .map(drop)
            } else {
                Err(InvocationError::Rpc(RpcError {
                    code: 400,
                    name: "CHAT_INVALID".to_string(),
                    value: None,
                }))
            }
        } else {
            Err(InvocationError::Rpc(RpcError {
                code: 400,
                name: "PEER_ID_INVALID".to_string(),
                value: None,
            }))
        }
    }
}
