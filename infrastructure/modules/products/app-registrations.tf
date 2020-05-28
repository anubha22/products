resource "azuread_application" "pars-upload" {
  name                       = var.pars_app_name
  reply_urls                 = concat(["https://${azurerm_cdn_endpoint.pars.host_name}"], var.add_local_pars_reply_url ? ["http://localhost:3000"] : [])
  group_membership_claims    = "SecurityGroup"
  homepage                   = "https://${azurerm_cdn_endpoint.pars.host_name}" #don't think we need this
  oauth2_allow_implicit_flow = true
  owners                     = var.app_registration_owners
  app_role {
    allowed_member_types = [
      "User",
    ]
    description  = "used to secure the pars upload endpoint via JWT"
    display_name = "pars_upload"
    is_enabled   = false
    value        = "id_token"
  }
}
