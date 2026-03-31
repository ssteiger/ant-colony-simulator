import {
  BellIcon,
  DatabaseIcon,
  FingerprintIcon,
  LogOutIcon,
  MoreVerticalIcon,
} from 'lucide-react'
import { useRouter } from '@tanstack/react-router'

import { Avatar, AvatarFallback, AvatarImage } from '~/lib/components/ui/avatar'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '~/lib/components/ui/dropdown-menu'
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from '~/lib/components/ui/sidebar'
import { toast } from 'sonner'
import { useMutation } from '@tanstack/react-query'
import { authClient } from '~/lib/auth-client'

export function NavUser({
  user,
}: {
  user: {
    name: string
    email: string
    avatar: string
  }
}) {
  const { isMobile } = useSidebar()
  const router = useRouter()

  const addPasskeyMutation = useMutation({
    mutationFn: async () => {
      const { error } = await authClient.passkey.addPasskey()
      if (error) throw new Error(error.message)
    },
    onSuccess: () => {
      toast.success('Passkey added successfully')
    },
    onError: (error) => {
      toast.error(`Failed to add passkey: ${error.message}`)
    },
  })

  const logOutMutation = useMutation({
    mutationFn: async () => {
      const { error } = await authClient.signOut()
      if (error) throw new Error(error.message)
    },
    onSuccess: () => {
      toast.success('Logout successful')
      router.navigate({ to: '/auth/login' })
    },
    onError: (error) => {
      toast.error(`Logout failed: ${error.message}`)
    },
  })

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton
              size="lg"
              className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            >
              <Avatar className="h-8 w-8 rounded-lg grayscale">
                <AvatarImage src={user.avatar} alt={user.name} />
                <AvatarFallback className="rounded-lg">CN</AvatarFallback>
              </Avatar>
              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-medium">{user.name}</span>
                <span className="truncate text-xs text-muted-foreground">{user.email}</span>
              </div>
              <MoreVerticalIcon className="ml-auto size-4" />
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className="w-[--radix-dropdown-menu-trigger-width] min-w-56 rounded-lg"
            side={isMobile ? 'bottom' : 'right'}
            align="end"
            sideOffset={4}
          >
            <DropdownMenuLabel className="p-0 font-normal">
              <div className="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
                <Avatar className="h-8 w-8 rounded-lg">
                  <AvatarImage src={user.avatar} alt={user.name} />
                  <AvatarFallback className="rounded-lg">CN</AvatarFallback>
                </Avatar>
                <div className="grid flex-1 text-left text-sm leading-tight">
                  <span className="truncate font-medium">{user.name}</span>
                  <span className="truncate text-xs text-muted-foreground">{user.email}</span>
                </div>
              </div>
            </DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuGroup>
              <DropdownMenuItem
                onClick={() => addPasskeyMutation.mutate()}
                disabled={addPasskeyMutation.isPending}
              >
                <FingerprintIcon />
                {addPasskeyMutation.isPending ? 'Adding Passkey ...' : 'Add Passkey'}
              </DropdownMenuItem>
              <DropdownMenuItem>
                <BellIcon />
                Notifications
              </DropdownMenuItem>
              <DropdownMenuItem asChild>
                <a
                  href="http://127.0.0.1:57323/project/default"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <DatabaseIcon />
                  Local DB
                </a>
              </DropdownMenuItem>
            </DropdownMenuGroup>
            <DropdownMenuSeparator />
            <DropdownMenuItem onClick={() => logOutMutation.mutate()}>
              <LogOutIcon />
              Log out
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  )
}
