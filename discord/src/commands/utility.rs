use serenity::framework::standard::macros::group;


group!({
  name: "Utility",
  options: {
  
  },
  commands: [cache_size, clear_cache, system_info],
});