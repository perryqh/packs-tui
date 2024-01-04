module Admin
  class Users
    def self.call
      UserManager::UserManagerThing.call
    end
  end
end