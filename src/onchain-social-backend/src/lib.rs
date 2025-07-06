use ic_cdk::api::time;
use ic_cdk_macros::*;
use candid::{CandidType, Deserialize, Principal};
use std::cell::RefCell;
use std::collections::HashMap;

const DEFAULT_AVATAR_URL: &str = "https://i.ibb.co/Pr3bY4X/default-avatar.png"; //Step 1: Default avatar

#[derive(CandidType, Deserialize, Clone)]
struct Post {
    id: u64,
    content: String,
    author: Principal,
    timestamp: u64,
    likes: Vec<Principal>,
    visibility: String,
}

#[derive(CandidType, Deserialize, Clone)]
struct UserProfile {
    user_principal: Principal,
    username: String,
    avatar_url: String,
    bio: String,
}

#[derive(CandidType, Deserialize, Clone)]
struct Comment {
    post_id: u64,
    commenter: Principal,
    text: String,
    timestamp: u64,
}

#[derive(CandidType, Deserialize)]
enum FollowResponse {
    Success,
    AlreadyFollowing,
    NotFollowing,
    CannotFollowSelf,
}

thread_local! {
    static POSTS: RefCell<HashMap<u64, Post>> = RefCell::new(HashMap::new());
    static NEXT_ID: RefCell<u64> = RefCell::new(0);
    static USERS: RefCell<HashMap<Principal, UserProfile>> = RefCell::new(HashMap::new());
    static COMMENTS: RefCell<Vec<Comment>> = RefCell::new(Vec::new());
    static FOLLOWERS: RefCell<HashMap<Principal, Vec<Principal>>> = RefCell::new(HashMap::new());
    static FOLLOWING: RefCell<HashMap<Principal, Vec<Principal>>> = RefCell::new(HashMap::new());
}

#[update]
fn create_post(content: String, visibility: String) -> Post {
    let author = ic_cdk::caller();
    let timestamp = time();

    let post = POSTS.with(|posts| {
        let mut posts = posts.borrow_mut();
        let id = NEXT_ID.with(|next| {
            let id = *next.borrow();
            *next.borrow_mut() += 1;
            id
        });

        let post = Post {
            id,
            content,
            author,
            timestamp,
            likes: vec![],
            visibility,
        };

        posts.insert(id, post.clone());
        post
    });

    post
}

#[query]
fn get_all_posts() -> Vec<Post> {
    let caller = ic_cdk::caller();
    POSTS.with(|posts| posts.borrow().values().filter(|post| post.visibility == "public" || post.author == caller).cloned().collect())
}

#[update]
fn register_user(username: String, avatar_url: String, bio: String) {
    let principal = ic_cdk::caller();
    let final_avatar = if avatar_url.trim().is_empty() {
        DEFAULT_AVATAR_URL.to_string()
    } else {
        avatar_url
    };

    let profile = UserProfile {
        user_principal: principal,
        username,
        avatar_url: final_avatar,
        bio,
    };

    USERS.with(|users| {
        users.borrow_mut().insert(principal, profile);
    });
}

#[query]
fn get_my_profile() -> Option<UserProfile> {
    let principal = ic_cdk::caller();
    USERS.with(|users| users.borrow().get(&principal).cloned())
}

#[update]
fn update_profile(username: String, avatar_url: String, bio: String) {
    let principal = ic_cdk::caller();

    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&principal) {
            user.username = username;
            user.avatar_url = if avatar_url.trim().is_empty() {
                DEFAULT_AVATAR_URL.to_string()
            } else {
                avatar_url
            };
            user.bio = bio;
        }
    });
}

#[update]
fn toggle_like_post(post_id: u64) -> Option<Post> {
    let caller = ic_cdk::caller();

    POSTS.with(|posts| {
        let mut posts = posts.borrow_mut();

        if let Some(post) = posts.get_mut(&post_id) {
            if post.likes.contains(&caller) {
                post.likes.retain(|p| p != &caller); // Unlike
            } else {
                post.likes.push(caller); // Like
            }
            return Some(post.clone());
        }

        None
    })
}

#[update]
fn add_comment(post_id: u64, text: String) {
    let commenter = ic_cdk::caller();
    let timestamp = time();

    let comment = Comment {
        post_id,
        commenter,
        text,
        timestamp,
    };

    COMMENTS.with(|comments| {
        comments.borrow_mut().push(comment);
    });
}

#[query]
fn get_comments(post_id: u64) -> Vec<Comment> {
    COMMENTS.with(|comments| {
        comments
            .borrow()
            .iter()
            .filter(|c| c.post_id == post_id)
            .cloned()
            .collect()
    })
}

#[update]
fn edit_post(post_id: u64, new_content: String) -> Option<Post> {
    let caller = ic_cdk::caller();
    POSTS.with(|posts| {
        let mut posts = posts.borrow_mut();
        if let Some(post) = posts.get_mut(&post_id) {
            if post.author == caller {
                post.content = new_content;
                Some(post.clone())
            } else {
                None // Unauthorized
            }
        } else {
            None // Post not found
        }
    })
}

#[update]
fn delete_post(post_id: u64) -> bool {
    let caller = ic_cdk::caller();
    POSTS.with(|posts| {
        let mut posts = posts.borrow_mut();
        if let Some(post) = posts.get(&post_id) {
            if post.author == caller {
                posts.remove(&post_id);
                true
            } else {
                false // Unauthorized
            }
        } else {
            false // Post not found
        }
    })
}

#[update]
fn follow_user(target: Principal) -> FollowResponse {
    let caller = ic_cdk::caller();
    if caller == target {
        return FollowResponse::CannotFollowSelf;
    }

    let mut already_following = false;

    FOLLOWING.with(|following| {
        let mut following_map = following.borrow_mut();
        let user_following = following_map.entry(caller).or_insert_with(Vec::new);
        if user_following.contains(&target) {
            already_following = true;
        } else {
            user_following.push(target);
        }
    });

    if already_following {
        return FollowResponse::AlreadyFollowing;
    }

    FOLLOWERS.with(|followers| {
        let mut followers_map = followers.borrow_mut();
        let user_followers = followers_map.entry(target).or_insert_with(Vec::new);
        user_followers.push(caller);
    });

    FollowResponse::Success
}

#[update]
fn unfollow_user(target: Principal) -> FollowResponse {
    let caller = ic_cdk::caller();

    let mut was_following = false;

    FOLLOWING.with(|following| {
        let mut following_map = following.borrow_mut();
        if let Some(user_following) = following_map.get_mut(&caller) {
            if let Some(pos) = user_following.iter().position(|p| p == &target) {
                user_following.remove(pos);
                was_following = true;
            }
        }
    });

    if !was_following {
        return FollowResponse::NotFollowing;
    }

    FOLLOWERS.with(|followers| {
        let mut followers_map = followers.borrow_mut();
        if let Some(user_followers) = followers_map.get_mut(&target) {
            user_followers.retain(|p| p != &caller);
        }
    });

    FollowResponse::Success
}

#[query]
fn get_following(user: Principal) -> Vec<Principal> {
    FOLLOWING.with(|following| {
        following.borrow().get(&user).cloned().unwrap_or_else(Vec::new)
    })
}

#[query]
fn get_followers(user: Principal) -> Vec<Principal> {
    FOLLOWERS.with(|followers| {
        followers.borrow().get(&user).cloned().unwrap_or_else(Vec::new)
    })
}

#[query]
fn get_feed() -> Vec<Post> {
    let caller = ic_cdk::caller();
    let mut feed = vec![];

    let following = FOLLOWING.with(|map| {
        map.borrow().get(&caller).cloned().unwrap_or_default()
    });

    POSTS.with(|posts| {
        let posts_map = posts.borrow();
        for post in posts_map.values() {
            if (post.author == caller || following.contains(&post.author))
            && post.visibility == "public"
            {
                feed.push(post.clone());
            }
        }
    });

    feed
}

#[query]
fn search_users(query: String) -> Vec<UserProfile> {
    let lowercase_query = query.to_lowercase();
    USERS.with(|users| {
        users
            .borrow()
            .values()
            .filter(|profile| profile.username.to_lowercase().contains(&lowercase_query))
            .cloned()
            .collect()
    })
}