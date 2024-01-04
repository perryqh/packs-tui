module Admin
  class PetManagerThing
     def self.call
       Dog::Bark.call
       Cat::Meow.call
     end
  end
end