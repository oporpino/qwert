#!/bin/bash

_create_user()
{
  user=$1
  password=$2

  echo "Creating user: $user"
  sudo dscl . -create /Users/$user 

  echo "Set default shel for user:  $user"
  sudo dscl . -create /Users/$user UserShell /bin/bash

  echo "Set name of user: $user"
  sudo dscl . -create /Users/$user RealName $user 

  echo "Create a unique id for user: $user"
  sudo dscl . -create /Users/$user UniqueID 1001 

  echo "Set primary group of user: $user"
  sudo dscl . -create /Users/$user PrimaryGroupID 1000 

  echo "Set home folder to user: $user"
  sudo mkdir /Users/$user
  sudo dscl . -create /Users/$user NFSHomeDirectory /Users/$user

  echo "Set Password"
  sudo dscl . -passwd /Users/$user $password

  echo "Enable remote login for: $user"
  sudo dseditgroup -o edit -a $user com.apple.access_ssh
}

_config_pair_user_home()
{
  user=$1

  echo "Config user configuraritions for paring: $user"
  # config bashrc
  sudo touch /Users/$user/.bashrc
  sudo echo "# call wemux and exit" >> /Users/$user/.bashrc
  sudo echo "exec wemux mirror" >> /Users/$user/.bashrc
  sudo echo "exit" >> /Users/$user/.bashrc

  #config profile
  sudo touch /Users/$user/.profile

  sudo echo 'eval $(/opt/homebrew/bin/brew shellenv)' >> /Users/$user/.profile
  sudo echo 'source $HOME/.bashrc' >> /Users/$user/.profile 
}

username=pair
password=paring

# install paring
id -u $username
if [[ $? != 0 ]] ; then
  echo "  - Start to install Paring"
  echo "  + - Create user for safety paring"
  sudo _create_user $username $password

  echo "  + - Config Pair user home"
  sudo _config_pair_user_home $username 
else
  echo "  - [warn] Paring is already installed. To reinstall please check the TMUX documentation."
fi

